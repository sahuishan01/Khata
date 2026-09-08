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
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.toRequestBody
import javax.inject.Inject

data class AuthUiState(val isChecking: Boolean = true, val isLoading: Boolean = false, val isLoggedIn: Boolean = false, val setupRequired: Boolean = false, val mustResetPassword: Boolean = false, val user: MeResponse? = null, val error: String? = null)
data class DashboardUiState(val stats: DashboardStats? = null, val analysis: AnalysisStats? = null, val isLoading: Boolean = false, val error: String? = null)
data class TxnFilter(val sortBy: String = "date", val sortDir: String = "desc", val category: String? = null, val from: String? = null, val to: String? = null, val preset: Int = 0, val search: String = "")
data class TxnUiState(val txns: TxnListResponse? = null, val categories: List<String> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class ChatUiState(val messages: List<ChatHistoryResponse> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class UsersUiState(val users: List<UserResponse> = emptyList(), val isLoading: Boolean = false, val error: String? = null, val success: String? = null)
data class AccountsUiState(val accounts: List<UserAccount> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class RulesUiState(val rules: List<CategoryRule> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class BudgetsUiState(val budgets: List<Budget> = emptyList(), val status: List<BudgetStatus> = emptyList(), val isLoading: Boolean = false, val error: String? = null)
data class PortfolioUiState(val snapshot: NetWorthSnapshot? = null, val isLoading: Boolean = false, val error: String? = null)
data class CategoriesUiState(val list: List<Category> = emptyList(), val isLoading: Boolean = false, val error: String? = null)

@HiltViewModel
class MainViewModel @Inject constructor(
    private val repository: KhataRepository,
    private val tokenManager: TokenManager,
    private val db: com.khata.app.data.KhataDatabase,
    private val syncEngine: com.khata.app.data.SyncEngine,
) : ViewModel() {
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

    fun updateServerUrl(url: String) {
        tokenManager.setServerUrl(url)
        _serverUrl.value = url
    }

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

    fun loadTransactions(sortBy: String = "date", sortDir: String = "desc", category: String? = null, from: String? = null, to: String? = null, preset: Int = 0) { viewModelScope.launch { try {
        _txnState.value = _txnState.value.copy(isLoading = true, error = null)
        _txnFilterState.value = TxnFilter(sortBy, sortDir, category, from, to, preset)
        val t = repository.listTxns(sortBy = sortBy, sortDir = sortDir, category = category, from = from, to = to)
        val txnCats = repository.listCategories()
        val managedCats = try { repository.listCategoriesV2().map { it.name } } catch (_: Exception) { emptyList() }
        val allCats = (txnCats + managedCats).distinct().sorted()
        _txnState.value = TxnUiState(txns = t, categories = allCats)
        _cachedTxns.value = t
    } catch (e: Exception) { _txnState.value = _txnState.value.copy(isLoading = false, error = e.message ?: "Failed") } }}

    fun toggleTransfer(id: String, v: Boolean) { viewModelScope.launch { try {
        repository.toggleTransfer(id, v);
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
    } catch (_: Exception) {} }}
    fun updateNotes(id: String, notes: String) { viewModelScope.launch { try {
        repository.updateNotes(id, notes);
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
    } catch (_: Exception) {} }}
    fun updateCategory(id: String, category: String) { viewModelScope.launch { try {
        repository.updateCategory(id, category)
        val current = _txnState.value
        current.txns?.let { list ->
            val updated = list.data.map { if (it.id == id) it.copy(category = category) else it }
            _txnState.value = current.copy(txns = list.copy(data = updated), error = null)
        }
    } catch (e: Exception) {
        _txnState.value = _txnState.value.copy(error = "Category update failed: ${e.message}")
    } }}
    fun createTxn(req: CreateTxnReq) { viewModelScope.launch { try {
        repository.createTxn(req);
        val f = _txnFilterState.value; loadTransactions(f.sortBy, f.sortDir, f.category, f.from, f.to, f.preset)
    } catch (_: Exception) {} }}

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
        repository.uploadStatement(part, pwPart, savePart)
        lastUploadUri = null
        onResult("Uploaded!")
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
    fun scanSmsInbox(context: Context, onResult: (String) -> Unit) { viewModelScope.launch(kotlinx.coroutines.Dispatchers.IO) {
        try {
            val cursor = context.contentResolver.query(
                android.provider.Telephony.Sms.Inbox.CONTENT_URI,
                arrayOf(
                    android.provider.Telephony.Sms.ADDRESS,
                    android.provider.Telephony.Sms.BODY,
                    android.provider.Telephony.Sms.DATE,
                ),
                null, null,
                "${android.provider.Telephony.Sms.DATE} DESC LIMIT 500",
            ) ?: return@launch onResult("Error: cannot read SMS")

            var found = 0
            val fmt = java.text.SimpleDateFormat("yyyy-MM-dd", java.util.Locale.US)
            cursor.use {
                val aI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.ADDRESS)
                val bI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.BODY)
                val dI = it.getColumnIndexOrThrow(android.provider.Telephony.Sms.DATE)
                while (it.moveToNext()) {
                    val body = it.getString(bI) ?: continue
                    val sender = it.getString(aI) ?: ""
                    val parsed = com.khata.app.sms.SmsParser.parse(body, sender) ?: continue
                    val date = fmt.format(java.util.Date(it.getLong(dI)))
                    val clientId = "sms_" + java.util.UUID.nameUUIDFromBytes(
                        (sender + "|" + parsed.amount + "|" + parsed.direction + "|" + date + "|" + (parsed.refNo ?: body.take(24)))
                            .toByteArray()
                    ).toString().take(16)
                    if (db.transactionDao().getByClientId(clientId) != null) continue
                    db.transactionDao().upsert(
                        com.khata.app.data.LocalTransaction(
                            clientId = clientId,
                            description = parsed.payee,
                            amount = parsed.amount,
                            direction = parsed.direction,
                            category = "Uncategorized",
                            bank = parsed.bank,
                            valueDate = date,
                            txnDate = date,
                            notes = "Auto-captured from SMS inbox (${parsed.bank})",
                            dirty = true,
                            pendingOp = "CREATE",
                        )
                    )
                    found++
                }
            }
            if (found > 0) syncEngine.sync()
            onResult(if (found == 0) "No new bank transactions found in SMS." else "Queued $found transaction(s) from SMS.")
        } catch (e: SecurityException) {
            onResult("Error: SMS permission not granted")
        } catch (e: Exception) {
            onResult("Error: ${e.message}")
        }
    } }

    private fun getFileName(context: Context, uri: Uri): String? {
        val c = context.contentResolver.query(uri, null, null, null, null)
        c?.use { if (it.moveToFirst()) { val i = it.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME); if (i >= 0) return it.getString(i) } }; return null
    }

    fun clearAllData(r: (String) -> Unit) { viewModelScope.launch { try { repository.clearAllData(); r("Cleared!") } catch (e: Exception) { r("Error: ${e.message}") } } }

    /** Load the current Gmail config so the UI can show whether a key is already stored. */
    fun loadEmailConfig(r: (UserEmailConfigResponse?) -> Unit) { viewModelScope.launch {
        try { r(repository.getEmailConfig()) } catch (_: Exception) { r(null) }
    } }

    /**
     * Send the Gmail credentials to the backend over HTTPS. The server encrypts the
     * app password at rest with AES-256-GCM (key derived per-user) before storing it;
     * nothing is persisted on the device. On success, kicks off an initial sync.
     */
    fun saveEmailConfig(req: SaveEmailConfigReq, r: (String) -> Unit) { viewModelScope.launch { try {
        repository.saveEmailConfig(req)
        val synced = try { repository.syncEmailNow(); " Sync started." } catch (_: Exception) { "" }
        r("Gmail connected — credentials encrypted on the server.$synced")
    } catch (e: Exception) { r("Error: ${e.message ?: "could not save Gmail config"}") } } }

    /** Most recent sync run, for the progress panel. */
    fun latestEmailRun(r: (EmailSyncRun?) -> Unit) { viewModelScope.launch {
        try { r(repository.latestEmailRun()) } catch (_: Exception) { r(null) }
    } }

    /** Trigger a run now; [r] gets a status line. */
    fun startEmailSync(r: (String) -> Unit) { viewModelScope.launch {
        try { repository.syncEmailNow(); r("Sync started") }
        catch (e: Exception) { r("Error: ${e.message ?: "could not start sync"}") }
    } }
}
