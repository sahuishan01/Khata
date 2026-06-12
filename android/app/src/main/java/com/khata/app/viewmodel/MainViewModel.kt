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

data class AuthUiState(val isChecking: Boolean = true, val isLoading: Boolean = false, val isLoggedIn: Boolean = false, val setupRequired: Boolean = false, val user: MeResponse? = null, val error: String? = null)
data class DashboardUiState(val stats: DashboardStats? = null, val analysis: AnalysisStats? = null, val isLoading: Boolean = false, val error: String? = null)
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
    private val repository: KhataRepository
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

    fun checkAuth() { viewModelScope.launch {
        try {
            val sr = repository.checkSetupStatus()
            if (!sr) { try { val u = repository.getMe(); _authState.value = AuthUiState(isChecking = false, isLoggedIn = true, user = u) } catch (_: Exception) { _authState.value = AuthUiState(isChecking = false) } }
            else { _authState.value = AuthUiState(isChecking = false, setupRequired = true) }
        } catch (_: Exception) { _authState.value = AuthUiState(isChecking = false, setupRequired = true) }
    }}

    fun login(email: String, password: String) { viewModelScope.launch { try {
        _authState.value = _authState.value.copy(isLoading = true, error = null)
        repository.login(email, password); val u = repository.getMe()
        _authState.value = AuthUiState(isChecking = false, isLoggedIn = true, user = u)
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

    fun loadTransactions(sortBy: String = "date", sortDir: String = "desc", category: String? = null, from: String? = null, to: String? = null) { viewModelScope.launch { try {
        _txnState.value = _txnState.value.copy(isLoading = true, error = null)
        val t = repository.listTxns(sortBy = sortBy, sortDir = sortDir, category = category, from = from, to = to)
        val c = repository.listCategories()
        _txnState.value = TxnUiState(txns = t, categories = c)
    } catch (e: Exception) { _txnState.value = _txnState.value.copy(isLoading = false, error = e.message ?: "Failed") } }}

    fun toggleTransfer(id: String, v: Boolean) { viewModelScope.launch { try { repository.toggleTransfer(id, v); loadTransactions() } catch (_: Exception) {} }}
    fun toggleInvestment(id: String, v: Boolean) { viewModelScope.launch { try { repository.toggleInvestment(id, v); loadTransactions() } catch (_: Exception) {} }}
    fun updateNotes(id: String, notes: String) { viewModelScope.launch { try { repository.updateNotes(id, notes); loadTransactions() } catch (_: Exception) {} }}
    fun createTxn(req: CreateTxnReq) { viewModelScope.launch { try { repository.createTxn(req); loadTransactions() } catch (_: Exception) {} }}

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

    fun resetPassword(c: String, n: String, onSuccess: () -> Unit) { viewModelScope.launch { try { repository.resetPassword(c, n); onSuccess() } catch (e: Exception) { _authState.value = _authState.value.copy(error = e.message) } }}

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

    fun uploadStatement(context: Context, uri: Uri, onResult: (String) -> Unit) { viewModelScope.launch { try {
        val ins = context.contentResolver.openInputStream(uri) ?: return@launch onResult("Error")
        val bytes = ins.readBytes(); ins.close()
        val name = getFileName(context, uri) ?: "upload_${System.currentTimeMillis()}"
        val part = MultipartBody.Part.createFormData("file", name, bytes.toRequestBody("application/octet-stream".toMediaTypeOrNull()))
        repository.uploadStatement(part); onResult("Uploaded!")
    } catch (e: Exception) { onResult("Error: ${e.message}") } }}

    private fun getFileName(context: Context, uri: Uri): String? {
        val c = context.contentResolver.query(uri, null, null, null, null)
        c?.use { if (it.moveToFirst()) { val i = it.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME); if (i >= 0) return it.getString(i) } }; return null
    }

    fun clearAllData(r: (String) -> Unit) { viewModelScope.launch { try { repository.clearAllData(); r("Cleared!") } catch (e: Exception) { r("Error: ${e.message}") } } }
}
