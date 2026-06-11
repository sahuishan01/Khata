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
    suspend fun login(email: String, password: String): String {
        val response = api.login(mapOf("email" to email, "password" to password))
        tokenManager.saveToken(response.token)
        return response.token
    }

    suspend fun setup(email: String, password: String): String {
        val response = api.setup(mapOf("email" to email, "password" to password))
        tokenManager.saveToken(response.token)
        return response.token
    }

    suspend fun checkSetupStatus(): Boolean = api.setupStatus().setupRequired

    suspend fun getMe(): MeResponse = api.me()

    suspend fun logout() = tokenManager.clearToken()

    suspend fun listUsers(): List<UserResponse> = api.listUsers()

    suspend fun createUser(email: String, password: String) =
        api.createUser(mapOf("email" to email, "password" to password))

    suspend fun deleteUser(id: String) = api.deleteUser(id)

    suspend fun resetPassword(current: String, newPassword: String) =
        api.resetPassword(mapOf("current_password" to current, "new_password" to newPassword))

    suspend fun getDashboard(): DashboardStats = api.dashboard()

    suspend fun getAnalysis(): AnalysisStats = api.analysis()

    suspend fun listTxns(
        page: Int = 1,
        category: String? = null,
        sortBy: String = "date",
        sortDir: String = "desc"
    ): TxnListResponse = api.listTxns(page = page, category = category, sortBy = sortBy, sortDir = sortDir)

    suspend fun listCategories(): List<String> = api.listCategories()

    suspend fun getChatHistory(): List<ChatHistoryResponse> = api.chatHistory()

    suspend fun askChat(question: String): ChatAskResponse = api.chatAsk(ChatAskRequest(question))

    suspend fun uploadStatement(part: MultipartBody.Part) = api.uploadStatement(part)
}
