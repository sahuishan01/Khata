package com.khata.app.data

import com.khata.app.api.*
import okhttp3.MultipartBody
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class KhataRepository @Inject constructor(
    private val api: KhataApi,
    private val tokenManager: TokenManager
) {
    suspend fun login(email: String, password: String): AuthResponse {
        val response = api.login(mapOf("email" to email, "password" to password))
        tokenManager.saveToken(response.token)
        return response
    }
    suspend fun setup(email: String, password: String): String {
        val r = api.setup(mapOf("email" to email, "password" to password))
        tokenManager.saveToken(r.token); return r.token
    }
    suspend fun checkSetupStatus(): Boolean = api.setupStatus().setupRequired
    suspend fun getMe(): MeResponse = api.me()
    suspend fun logout() = tokenManager.clearToken()

    suspend fun listUsers(): List<UserResponse> = api.listUsers()
    suspend fun createUser(e: String, p: String) = api.createUser(mapOf("email" to e, "password" to p))
    suspend fun deleteUser(id: String) = api.deleteUser(id)
    suspend fun resetPassword(c: String, n: String) = api.resetPassword(mapOf("current_password" to c, "new_password" to n))
    suspend fun updateEmail(email: String) = api.updateEmail(mapOf("email" to email))

    suspend fun getDashboard(): DashboardStats = api.dashboard()
    suspend fun getAnalysis(): AnalysisStats = api.analysis()

    suspend fun listTxns(page: Int = 1, category: String? = null, sortBy: String = "date", sortDir: String = "desc", from: String? = null, to: String? = null): TxnListResponse =
        api.listTxns(page = page, category = category, sortBy = sortBy, sortDir = sortDir, from = from, to = to)
    suspend fun listCategories(): List<String> = api.listCategories()
    suspend fun createTxn(req: CreateTxnReq) = api.createTxn(req)
    suspend fun toggleTransfer(id: String, v: Boolean) = api.toggleTransfer(id, ToggleTransferReq(v))
    suspend fun updateNotes(id: String, notes: String) = api.updateNotes(id, UpdateNotesReq(notes))
    suspend fun updateCategory(id: String, category: String) = api.updateCategory(id, mapOf("category" to category, "scope" to "single"))

    suspend fun listAccounts() = api.listAccounts()
    suspend fun createAccount(label: String, identifier: String) = api.createAccount(CreateAccountReq(label, identifier))
    suspend fun deleteAccount(id: String) = api.deleteAccount(id)

    suspend fun listRules() = api.listRules()
    suspend fun createRule(pattern: String, category: String) = api.createRule(CreateRuleReq(pattern, category))
    suspend fun deleteRule(id: String) = api.deleteRule(id)
    suspend fun applyRules() = api.applyRules()

    suspend fun listBudgets() = api.listBudgets()
    suspend fun createBudget(category: String, limit: Double) = api.createBudget(CreateBudgetReq(category, limit))
    suspend fun deleteBudget(id: String) = api.deleteBudget(id)
    suspend fun budgetStatus() = api.budgetStatus()

    suspend fun portfolioSnapshot() = api.portfolioSnapshot()
    suspend fun createAsset(name: String, type: String, value: Double) = api.createAsset(mapOf("name" to name, "asset_type" to type, "value" to value))
    suspend fun deleteAsset(id: String) = api.deleteAsset(id)
    suspend fun createLiability(name: String, type: String, value: Double) = api.createLiability(mapOf("name" to name, "liability_type" to type, "value" to value))
    suspend fun deleteLiability(id: String) = api.deleteLiability(id)

    suspend fun uploadStatement(part: MultipartBody.Part) = api.uploadStatement(part)
    suspend fun clearAllData() = api.clearAllData()

    suspend fun getChatHistory(): List<ChatHistoryResponse> = api.chatHistory()
    suspend fun askChat(question: String): ChatAskResponse = api.chatAsk(ChatAskRequest(question))

    suspend fun listCategoriesV2() = api.listCategoriesV2()
    suspend fun createCategory(name: String, type: String, color: String?, desc: String?) = api.createCategory(CreateCategoryReq(name, type, color, desc))
    suspend fun deleteCategory(id: String) = api.deleteCategory(id)
}
