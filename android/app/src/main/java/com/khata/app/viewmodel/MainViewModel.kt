package com.khata.app.viewmodel

import android.content.Context
import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.khata.app.api.*
import com.khata.app.data.KhataRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.coroutineScope
import com.khata.app.sms.SmsImportProgress
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.toRequestBody
import javax.inject.Inject

data class AuthUiState(val isChecking: Boolean = true, val isLoading: Boolean = false, val isLoggedIn: Boolean = false, val setupRequired: Boolean = false, val mustResetPassword: Boolean = false, val user: MeResponse? = null, val error: String? = null)
data class DashboardUiState(val stats: DashboardStats? = null, val analysis: AnalysisStats? = null, val isLoading: Boolean = false, val error: String? = null)
data class TxnFilter(val sortBy: String = "date", val sortDir: String = "desc", val category: String? = null, val from: String? = null, val to: String? = null, val preset: Int = 0, val search: String = "", val duplicatesOnly: Boolean = false)
data class TxnUiState(val txns: TxnListResponse? = null, val categories: List<String> = emptyList(), val isLoading: Boolean = false, val error: String? = null, val isLoadingMore: Boolean = false, val hasMore: Boolean = false, val moreError: String? = null)
data class ChatUiState(val messages: List<ChatHistoryResponse> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class UsersUiState(val users: List<UserResponse> = emptyList(), val isLoading: Boolean = false, val error: String? = null, val success: String? = null)
data class AccountsUiState(val accounts: List<UserAccount> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class RulesUiState(val rules: List<CategoryRule> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class BudgetsUiState(val budgets: List<Budget> = emptyList(), val status: List<BudgetStatus> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class PortfolioUiState(val snapshot: NetWorthSnapshot? = null, val isLoading: Boolean = false, val error: String? = null)
data class CategoriesUiState(val list: List<Category> = emptyList(), val isLoading: Boolean = false, val error: String? = null)

@HiltViewModel
@OptIn(kotlinx.coroutines.FlowPreview::class)
class MainViewModel @Inject constructor(
    private val repository: KhataRepository,
    private val tokenManager: TokenManager,
    private val db: com.khata.app.data.KhataDatabase,
    private val syncEngine: com.khata.app.data.SyncEngine,
    private val smsDecisions: com.khata.app.sms.SmsDecisionPipeline,
) : ViewModel() {
    private val _smsProgress = MutableStateFlow(SmsImportProgress())
    val smsProgress: StateFlow<SmsImportProgress> = _smsProgress.asStateFlow()
    private val _smsDecisionJobs = MutableStateFlow<List<com.khata.app.sms.SmsDecisionJob>>(emptyList())
    val smsDecisionJobs: StateFlow<List<com.khata.app.sms.SmsDecisionJob>> = _smsDecisionJobs.asStateFlow()
    private val _smsDecisionSummary = MutableStateFlow(com.khata.app.sms.SmsDecisionSummary())
    val smsDecisionSummary = _smsDecisionSummary.asStateFlow()
    fun retrySmsProcessing() { smsDecisions.resume() }
    fun retrySmsReview() { viewModelScope.launch(Dispatchers.IO) { smsDecisions.retryReview() } }
    fun ignoreSmsReview(id: String) { viewModelScope.launch(Dispatchers.IO) { smsDecisions.ignore(id) } }
    fun resolveSmsReview(id: String, amount: Double, direction: String, done: (String?) -> Unit) {
        viewModelScope.launch {
            val error = withContext(Dispatchers.IO) { smsDecisions.resolve(id,amount,direction) }
            if (error == null) { loadTransactions(); refreshDashboard() }
            done(error)
        }
    }
    private val _authState = MutableStateFlow(AuthUiState()); val authState: StateFlow<AuthUiState> = _authState.asStateFlow()
    private val _dashboardState = MutableStateFlow(DashboardUiState()); val dashboardState: StateFlow<DashboardUiState> = _dashboardState.asStateFlow()
    private val _txnState = MutableStateFlow(TxnUiState()); val txnState: StateFlow<TxnUiState> = _txnState.asStateFlow()
    private val _chatState = MutableStateFlow(ChatUiState()); val chatState: StateFlow<ChatUiState> = _chatState.asStateFlow()
    private val _usersState = MutableStateFlow(UsersUiState()); val usersState: StateFlow<UsersUiState> = _usersState.asStateFlow()
    private val _accountsState = MutableStateFlow(AccountsUiState()); val accountsState: StateFlow<AccountsUiState> = _accountsState.asStateFlow()
    private val _rulesState = MutableStateFlow(RulesUiState()); val rulesState: StateFlow<RulesUiState> = _rulesState.asStateFlow()
    private val _budgetsState = MutableStateFlow(BudgetsUiState()); val budgetsState: StateFlow<BudgetsUiState> = _budgetsState.asStateFlow()
    private val _portfolioState = MutableStateFlow(PortfolioUiState()); val portfolioState: StateFlow<PortfolioUiState> = _portfolioState.asStateFlow()
    private val _categoriesState = MutableStateFlow(CategoriesUiState()); val categoriesState: StateFlow<CategoriesUiState> = _categoriesState.asStateFlow()
    private val _txnFilterState = MutableStateFlow(TxnFilter()); val txnFilterState: StateFlow<TxnFilter> = _txnFilterState.asStateFlow()
    private val _cachedTxns = MutableStateFlow<TxnListResponse?>(null); val cachedTxns: StateFlow<TxnListResponse?> = _cachedTxns.asStateFlow()
    private val _serverUrl = MutableStateFlow(tokenManager.getServerUrl()); val serverUrl: StateFlow<String> = _serverUrl.asStateFlow()
    val deviceSyncState = syncEngine.state
    val lastDeviceSync = syncEngine.lastSuccess
    val pendingSyncCount = db.transactionDao().pendingCountFlow()
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), 0)

    init {
        viewModelScope.launch {
            combine(_authState,tokenManager.tokenFlow,tokenManager.serverUrlFlow) { auth,token,url ->
                if (auth.isLoggedIn && !auth.mustResetPassword) com.khata.app.sms.smsSession(token,url)?.owner else null
            }.distinctUntilChanged().collectLatest { owner ->
                _smsDecisionJobs.value = emptyList()
                _smsDecisionSummary.value = com.khata.app.sms.SmsDecisionSummary()
                if (owner != null) {
                    smsDecisions.resume()
                    coroutineScope {
                        launch { smsDecisions.active(owner).collect { _smsDecisionJobs.value = it } }
                        launch { smsDecisions.summary(owner).collect { _smsDecisionSummary.value = it } }
                        launch {
                            smsDecisions.summary(owner).map { it.accepted }.filter { it > 0 }.distinctUntilChanged()
                                .debounce(750).conflate().collect {
                                    transactionPager.refresh(_txnFilterState.value)
                                    refreshDashboard()
                                }
                        }
                    }
                }
            }
        }
        viewModelScope.launch {
            syncEngine.lastSuccess.filterNotNull().collect {
                if (_authState.value.isLoggedIn && !_authState.value.mustResetPassword) {
                    val f = _txnFilterState.value
                    loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
                    refreshDashboard()
                }
            }
        }
    }

    /** Called only while the signed-in app is in the foreground. */
    suspend fun syncPendingTransactions(force: Boolean = false) {
        if (!_authState.value.isLoggedIn || _authState.value.mustResetPassword || _smsProgress.value.running) return
        withContext(Dispatchers.IO) {
            if (force || db.transactionDao().getOutbox().isNotEmpty()) syncEngine.sync()
        }
    }

    fun retryDeviceSync() {
        if (_smsProgress.value.running || syncEngine.state.value is com.khata.app.data.SyncUiState.Syncing) return
        viewModelScope.launch(Dispatchers.IO) { syncEngine.sync() }
    }

    fun updateServerUrl(url: String) {
        tokenManager.setServerUrl(url)
        _serverUrl.value = url
    }

    private val _lastExcludedDuplicateId = MutableStateFlow<String?>(null)
    val lastExcludedDuplicateId: StateFlow<String?> = _lastExcludedDuplicateId.asStateFlow()

    fun updateTxnFilter(filter: TxnFilter) { _txnFilterState.value = filter }

    fun checkAuth() { viewModelScope.launch {
        try {
            val sr = repository.checkSetupStatus()
            if (!sr) { try { val u = repository.getMe(); _authState.value = AuthUiState(isChecking = false, isLoggedIn = true, mustResetPassword = u.mustResetPassword, user = if (u.mustResetPassword) null else u) } catch (_: Exception) { _authState.value = AuthUiState(isChecking = false) } }
            else { _authState.value = AuthUiState(isChecking = false, setupRequired = true) }
        } catch (_: Exception) { _authState.value = AuthUiState(isChecking = false, setupRequired = true) }
    }}

    fun login(email: String, password: String) { viewModelScope.launch { try {
        _authState.value = _authState.value.copy(isLoading = true, error = null)
        val authResp = repository.login(email, password)
        val mustReset = authResp.mustResetPassword
        val u = if (!mustReset) repository.getMe() else null
        _authState.value = AuthUiState(isChecking = false, isLoggedIn = true, mustResetPassword = mustReset, user = u)
    } catch (e: Exception) { _authState.value = _authState.value.copy(isLoading = false, error = e.message ?: "Login failed") } }}

    fun setup(email: String, password: String) { viewModelScope.launch { try {
        _authState.value = _authState.value.copy(isLoading = true, error = null)
        repository.setup(email, password); val u = repository.getMe()
        _authState.value = AuthUiState(isChecking = false, isLoggedIn = true, user = u)
    } catch (e: Exception) { _authState.value = _authState.value.copy(isLoading = false, error = e.message ?: "Setup failed") } }}

    fun logout() { viewModelScope.launch { repository.logout(); _authState.value = AuthUiState(isChecking = false) }}

    fun refreshDashboard() { viewModelScope.launch { try {
        _dashboardState.value = _dashboardState.value.copy(isLoading = true, error = null)
        val s = repository.getDashboard(); val a = repository.getAnalysis()
        _dashboardState.value = DashboardUiState(stats = s, analysis = a)
    } catch (e: Exception) { _dashboardState.value = _dashboardState.value.copy(isLoading = false, error = e.message) } }}

    private val transactionPager = TransactionPager { page, f ->
        repository.listTxns(page = page, sortBy = f.sortBy, sortDir = f.sortDir, category = f.category,
            from = f.from, to = f.to, duplicatesOnly = f.duplicatesOnly, search = f.search.ifBlank { null })
    }
    init {
        viewModelScope.launch {
            transactionPager.state.collect { p ->
                _txnState.value = _txnState.value.copy(txns = p.list, isLoading = p.loading, error = p.error,
                    isLoadingMore = p.loadingMore, hasMore = p.hasMore, moreError = p.moreError)
                if (p.list != null) _cachedTxns.value = p.list
            }
        }
    }
    fun loadMoreTransactions() { viewModelScope.launch { transactionPager.loadMore() } }
    fun loadTransactions(sortBy: String = "date", sortDir: String = "desc", category: String? = null, from: String? = null, to: String? = null, preset: Int = 0) {
        val query = _txnFilterState.value.copy(sortBy = sortBy, sortDir = sortDir, category = category, from = from, to = to, preset = preset)
        _txnFilterState.value = query
        viewModelScope.launch {
            transactionPager.refresh(query)
            try {
                val txnCats = repository.listCategories()
                val managed = try { repository.listCategoriesV2().map { it.name } } catch (_: Exception) { emptyList() }
                _txnState.value = _txnState.value.copy(categories = (txnCats + managed).distinct().sorted())
            } catch (_: Exception) { }
        }
    }

    fun excludeDuplicate(id: String) { viewModelScope.launch { try {
        repository.excludeDuplicate(id)
        _lastExcludedDuplicateId.value = id
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
        refreshDashboard()
    } catch (_: Exception) { _txnState.value = _txnState.value.copy(error = "Could not exclude duplicate. Refresh and retry.") } } }
    fun restoreDuplicate() { viewModelScope.launch { try {
        val id = _lastExcludedDuplicateId.value ?: return@launch
        repository.restoreDuplicate(id)
        _lastExcludedDuplicateId.value = null
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
        refreshDashboard()
    } catch (_: Exception) { _txnState.value = _txnState.value.copy(error = "Could not restore transaction. Please retry.") } } }

    fun toggleTransfer(id: String, v: Boolean) { viewModelScope.launch { try {
        repository.toggleTransfer(id, v);
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
    } catch (_: Exception) {} }}
    suspend fun updateTitle(id: String, title: String): String? {
        val clean = title.trim()
        if (clean.isEmpty() || clean.length > 500) return "Title must contain 1–500 characters."
        return try {
            repository.updateTitle(id, clean)
            val f = _txnFilterState.value
            loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
            refreshDashboard()
            null
        } catch (e: CancellationException) { throw e
        } catch (_: Exception) { "Could not save the title. Check your connection and retry." }
    }
    suspend fun updateTransaction(id: String, req: UpdateTxnReq): String? {
        return try {
            repository.updateTxn(id, req)
            val f = _txnFilterState.value
            loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
            refreshDashboard()
            null
        } catch (e: CancellationException) { throw e
        } catch (e: retrofit2.HttpException) {
            if (e.code() == 409) "This transaction changed. Reopen it to review the latest values before saving."
            else "Could not save the transaction. Check your connection and retry."
        } catch (_: Exception) { "Could not save the transaction. Check your connection and retry." }
    }
    fun updateNotes(id: String, notes: String) { viewModelScope.launch { try {
        repository.updateNotes(id, notes);
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
    } catch (_: Exception) {} }}
    fun updateCategory(id: String, category: String) { viewModelScope.launch { try {
        repository.updateCategory(id, category)
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
    } catch (e: Exception) {
        _txnState.value = _txnState.value.copy(error = "Category update failed: ${e.message}")
    } }}
    fun createTxn(req: CreateTxnReq, onFinished: (String?) -> Unit = {}) { viewModelScope.launch { try {
        repository.createTxn(req);
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
        refreshDashboard()
        onFinished(null)
    } catch (e: Exception) { onFinished(e.message ?: "Could not save transaction. Please try again.") } }}

    fun loadChatHistory() { viewModelScope.launch { try {
        _chatState.value = _chatState.value.copy(isLoading = true, error = null)
        _chatState.value = ChatUiState(messages = repository.getChatHistory())
    } catch (e: Exception) { _chatState.value = _chatState.value.copy(isLoading = false, error = e.message ?: "Failed") } }}

    fun sendChatMessage(question: String) { viewModelScope.launch {
        val tmp = ChatHistoryResponse(System.currentTimeMillis().toString(), "user", question, null)
        _chatState.value = _chatState.value.copy(messages = _chatState.value.messages + tmp, isLoading = true, error = null)
        try {
            val r = repository.askChat(question)
            val reply = ChatHistoryResponse((System.currentTimeMillis() + 1).toString(), "assistant", r.answer, r.sqlUsed)
            _chatState.value = _chatState.value.copy(messages = _chatState.value.messages + reply, isLoading = false)
        } catch (e: Exception) {
            val err = ChatHistoryResponse((System.currentTimeMillis() + 1).toString(), "assistant", "Error: ${e.message}", null)
            _chatState.value = _chatState.value.copy(messages = _chatState.value.messages + err, isLoading = false, error = e.message)
        }
    }}

    fun loadUsers() { viewModelScope.launch { try { _usersState.value = UsersUiState(users = repository.listUsers()) } catch (e: Exception) { _usersState.value = _usersState.value.copy(error = e.message) } }}
    fun createUser(e: String, p: String) { viewModelScope.launch { try { repository.createUser(e, p); loadUsers(); _usersState.value = _usersState.value.copy(success = "User created") } catch (ex: Exception) { _usersState.value = _usersState.value.copy(error = ex.message) } }}
    fun deleteUser(id: String) { viewModelScope.launch { try { repository.deleteUser(id); loadUsers() } catch (_: Exception) {} }}

    fun resetPassword(c: String, n: String, onSuccess: () -> Unit) { viewModelScope.launch { try {
        repository.resetPassword(c, n)
        _authState.value = _authState.value.copy(mustResetPassword = false, isLoading = false)
        onSuccess()
    } catch (e: Exception) { _authState.value = _authState.value.copy(error = e.message) } }}
    fun updateEmail(email: String) { viewModelScope.launch { try { repository.updateEmail(email); val u = repository.getMe(); _authState.value = _authState.value.copy(user = u) } catch (_: Exception) {} }}

    fun loadAccounts() { viewModelScope.launch { try { _accountsState.value = AccountsUiState(accounts = repository.listAccounts()) } catch (e: Exception) { _accountsState.value = _accountsState.value.copy(error = e.message) } }}
    fun createAccount(l: String, i: String) { viewModelScope.launch { try { repository.createAccount(l, i); loadAccounts() } catch (e: Exception) { _accountsState.value = _accountsState.value.copy(error = e.message) } }}
    fun deleteAccount(id: String) { viewModelScope.launch { try { repository.deleteAccount(id); loadAccounts() } catch (_: Exception) {} }}

    fun loadRules() { viewModelScope.launch { try { _rulesState.value = RulesUiState(rules = repository.listRules()) } catch (e: Exception) { _rulesState.value = _rulesState.value.copy(error = e.message) } }}
    fun createRule(p: String, c: String) { viewModelScope.launch { try { repository.createRule(p, c); loadRules() } catch (e: Exception) { _rulesState.value = _rulesState.value.copy(error = e.message) } }}
    fun deleteRule(id: String) { viewModelScope.launch { try { repository.deleteRule(id); loadRules() } catch (_: Exception) {} }}
    fun applyRules() { viewModelScope.launch { try { repository.applyRules(); } catch (_: Exception) {} }}

    fun loadBudgets() { viewModelScope.launch { try {
        val b = repository.listBudgets(); val s = repository.budgetStatus()
        _budgetsState.value = BudgetsUiState(budgets = b, status = s)
    } catch (e: Exception) { _budgetsState.value = _budgetsState.value.copy(error = e.message) } }}
    fun createBudget(c: String, l: Double) { viewModelScope.launch { try { repository.createBudget(c, l); loadBudgets() } catch (e: Exception) { _budgetsState.value = _budgetsState.value.copy(error = e.message) } }}
    fun deleteBudget(id: String) { viewModelScope.launch { try { repository.deleteBudget(id); loadBudgets() } catch (_: Exception) {} }}

    fun loadPortfolio() { viewModelScope.launch { try { _portfolioState.value = PortfolioUiState(snapshot = repository.portfolioSnapshot()) } catch (e: Exception) { _portfolioState.value = _portfolioState.value.copy(error = e.message) } }}
    fun createAsset(n: String, t: String, v: Double) { viewModelScope.launch { try { repository.createAsset(n, t, v); loadPortfolio() } catch (e: Exception) { _portfolioState.value = _portfolioState.value.copy(error = e.message) } }}
    fun deleteAsset(id: String) { viewModelScope.launch { try { repository.deleteAsset(id); loadPortfolio() } catch (_: Exception) {} }}
    fun createLiability(n: String, t: String, v: Double) { viewModelScope.launch { try { repository.createLiability(n, t, v); loadPortfolio() } catch (e: Exception) { _portfolioState.value = _portfolioState.value.copy(error = e.message) } }}
    fun deleteLiability(id: String) { viewModelScope.launch { try { repository.deleteLiability(id); loadPortfolio() } catch (_: Exception) {} }}

    fun loadCategories() { viewModelScope.launch { try { _categoriesState.value = CategoriesUiState(list = repository.listCategoriesV2()) } catch (e: Exception) { _categoriesState.value = _categoriesState.value.copy(error = e.message) } }}
    fun createCategory(n: String, t: String, c: String?, d: String?) { viewModelScope.launch { try { repository.createCategory(n, t, c, d); loadCategories() } catch (e: Exception) { _categoriesState.value = _categoriesState.value.copy(error = e.message) } }}
    fun deleteCategory(id: String) { viewModelScope.launch { try { repository.deleteCategory(id); loadCategories() } catch (_: Exception) {} }}

    /** Last file the user picked, kept so a password prompt can retry it. */
    var lastUploadUri: Uri? = null
        private set

    /**
     * Upload a statement. `onResult` receives one of:
     *  - "Uploaded!" / a bank summary on success
     *  - "PASSWORD_REQUIRED" — the PDF is locked and needs a password
     *  - "PASSWORD_INCORRECT" — the supplied password was wrong
     *  - "Error: <message>" — anything else
     */
    fun uploadStatement(
        context: Context,
        uri: Uri,
        password: String? = null,
        savePassword: Boolean = true,
        onResult: (String) -> Unit,
    ) { viewModelScope.launch { try {
        lastUploadUri = uri
        val ins = context.contentResolver.openInputStream(uri) ?: return@launch onResult("Error: could not open file")
        val bytes = ins.readBytes(); ins.close()
        val name = getFileName(context, uri) ?: "upload_${System.currentTimeMillis()}"
        val part = MultipartBody.Part.createFormData("file", name, bytes.toRequestBody("application/octet-stream".toMediaTypeOrNull()))
        val pwPart = password?.takeIf { it.isNotBlank() }
            ?.toRequestBody("text/plain".toMediaTypeOrNull())
        val savePart = if (pwPart != null) {
            (if (savePassword) "true" else "false").toRequestBody("text/plain".toMediaTypeOrNull())
        } else null
        val res = repository.uploadStatement(part, pwPart, savePart)
        lastUploadUri = null
        val warnings = (res["warnings"] as? List<*>)?.mapNotNull { it?.toString() }.orEmpty()
        onResult(if (warnings.isNotEmpty()) "Uploaded! ${warnings.joinToString(" ")}" else "Uploaded!")
    } catch (e: retrofit2.HttpException) {
        val body = e.response()?.errorBody()?.string().orEmpty()
        val code = Regex("\"code\"\\s*:\\s*\"([^\"]+)\"").find(body)?.groupValues?.get(1)
        val msg = Regex("\"error\"\\s*:\\s*\"([^\"]+)\"").find(body)?.groupValues?.get(1)
        when (code) {
            "pdf_password_required" -> onResult("PASSWORD_REQUIRED")
            "pdf_password_incorrect" -> onResult("PASSWORD_INCORRECT")
            else -> onResult("Error: ${msg ?: e.message()}")
        }
    } catch (e: Exception) { onResult("Error: ${e.message}") } }}

    /**
     * Backfill: read the SMS inbox, parse bank transaction texts and queue any
     * new ones for the server (real-time capture already handles new SMS).
     * Requires READ_SMS. `onResult` gets a short summary or an error.
     */
    fun scanSmsInbox(context: Context, onResult: (String) -> Unit) {
        if (_smsProgress.value.running) return
        _smsProgress.value = SmsImportProgress(running = true, phase = "Opening SMS inbox…")
        viewModelScope.launch(Dispatchers.IO) {
            var progress = _smsProgress.value
            try {
                progress = progress.copy(phase = "Checking server history…")
                _smsProgress.value = progress
                try {
                    syncEngine.refreshCacheForSmsScan()
                } catch (e: CancellationException) {
                    throw e
                } catch (_: Exception) {
                    // Offline import remains available; never discard queued work.
                    progress = progress.copy(serverCheckFailed = true)
                }
                val cursor = context.contentResolver.query(
                    android.provider.Telephony.Sms.Inbox.CONTENT_URI,
                    arrayOf(android.provider.Telephony.Sms.ADDRESS, android.provider.Telephony.Sms.BODY, android.provider.Telephony.Sms.DATE, android.provider.Telephony.Sms.DATE_SENT),
                    null, null, "${android.provider.Telephony.Sms.DATE} DESC",
                ) ?: error("Cannot read SMS inbox")
                cursor.use {
                    progress = progress.copy(total = it.count, phase = "Scanning SMS inbox…")
                    _smsProgress.value = progress
                    val aI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.ADDRESS)
                    val bI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.BODY)
                    val dI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.DATE)
                    val sI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.DATE_SENT)
                    while (it.moveToNext()) {
                        kotlinx.coroutines.currentCoroutineContext().ensureActive()
                        progress = progress.copy(scanned = progress.scanned + 1)
                        val body = it.getString(bI).orEmpty()
                        if (!com.khata.app.sms.SmsCandidateFilter.isCandidate(body)) {
                            progress = progress.copy(filtered=progress.filtered+1)
                            continue
                        }
                        val sender = it.getString(aI).orEmpty()
                        val arrival = it.getLong(dI)
                        val timestamp = com.khata.app.sms.SmsTimestamps.fromInbox(arrival,it.getLong(sI))
                        if (smsDecisions.enqueue(body, sender, timestamp, scheduleNow=false,legacyTimestamp=arrival)) {
                            progress = progress.copy(matched = progress.matched + 1)
                        } else progress = progress.copy(alreadySaved = progress.alreadySaved + 1)
                        if (progress.scanned % 25 == 0 || progress.scanned == progress.total) _smsProgress.value = progress
                    }
                }
                smsDecisions.resume()
                val message = "${progress.matched} monetary SMS queued for classification; ${progress.filtered} skipped on this device. Queued does not mean imported. See SMS classification progress below; uncertain messages need review."
                _smsProgress.value = progress.copy(running=false,phase="Queued",message=message)
                withContext(Dispatchers.Main) { onResult(message) }
            } catch (e: CancellationException) {
                _smsProgress.value = progress.copy(running = false, phase = "Stopped", message = "Scan stopped. Saved transactions remain queued.")
                throw e
            } catch (e: Exception) {
                val message = if (e is SecurityException) "SMS permission is not granted. Allow SMS access in Android settings, then retry."
                    else "SMS import failed: ${e.message ?: "Unknown error"}. Saved transactions remain on this device."
                _smsProgress.value = progress.copy(running = false, phase = "Failed", message = message, failed = true)
                withContext(Dispatchers.Main) { onResult("Error: $message") }
            }
        }
    }

    private fun getFileName(context: Context, uri: Uri): String? {
        val c = context.contentResolver.query(uri, null, null, null, null)
        c?.use { if (it.moveToFirst()) { val i = it.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME); if (i >= 0) return it.getString(i) } }; return null
    }

    fun clearAllData(r: (String) -> Unit) {
        if (_smsProgress.value.running) { r("Wait for the SMS import to finish before clearing data."); return }
        viewModelScope.launch {
            try {
                syncEngine.clearAllData()
                _smsProgress.value = SmsImportProgress()
                loadTransactions()
                refreshDashboard()
                r("Server history and this device's saved and queued transactions cleared. You can scan your SMS inbox again.")
            } catch (e: CancellationException) { throw e }
            catch (e: Exception) { r("Error: ${e.message}") }
        }
    }

    /** Load the current Gmail config so the UI can show whether a key is already stored. */
    fun loadEmailConfig(r: (UserEmailConfigResponse?, String?) -> Unit) { viewModelScope.launch {
        try { r(repository.getEmailConfig(), null) } catch (e: Exception) { r(null, "Could not load email connection: ${e.message ?: "connection failed"}") }
    } }

    /**
     * Send the Gmail credentials to the backend over HTTPS. The server encrypts the
     * app password at rest with AES-256-GCM (key derived per-user) before storing it;
     * nothing is persisted on the device. On success, kicks off an initial sync.
     */
    fun saveEmailConfig(req: SaveEmailConfigReq, r: (String) -> Unit) { viewModelScope.launch { try {
        repository.saveEmailConfig(req)
        val synced = try { repository.syncEmailNow(); " Sync started." } catch (e: Exception) { " Configuration saved, but sync could not start: ${e.message ?: "please retry"}." }
        r("Gmail connected — credentials encrypted on the server.$synced")
    } catch (e: Exception) { r("Error: ${e.message ?: "could not save Gmail config"}") } } }

    /** Most recent sync run, for the progress panel. */
    fun latestEmailRun(r: (EmailSyncRun?, String?) -> Unit) { viewModelScope.launch {
        try { r(repository.latestEmailRun(), null) } catch (e: Exception) { r(null, "Could not refresh sync progress: ${e.message ?: "connection failed"}") }
    } }

    /** Trigger a run now; [r] gets a status line. */
    fun startEmailSync(r: (String) -> Unit) { viewModelScope.launch {
        try { repository.syncEmailNow(); r("Sync started") }
        catch (e: Exception) { r("Error: ${e.message ?: "could not start sync"}") }
    } }
}
