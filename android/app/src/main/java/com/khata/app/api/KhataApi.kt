package com.khata.app.api

import okhttp3.MultipartBody
import retrofit2.http.*

interface KhataApi {
    @POST("api/auth/login")
    suspend fun login(@Body body: Map<String, String>): AuthResponse

    @POST("api/auth/setup")
    suspend fun setup(@Body body: Map<String, String>): AuthResponse

    @GET("api/auth/setup-status")
    suspend fun setupStatus(): SetupStatusResponse

    @GET("api/auth/me")
    suspend fun me(): MeResponse

    @GET("api/auth/users") suspend fun listUsers(): List<UserResponse>
    @POST("api/auth/users") suspend fun createUser(@Body body: Map<String, String>): UserResponse
    @DELETE("api/auth/users/{id}") suspend fun deleteUser(@Path("id") id: String): MessageResponse
    @POST("api/auth/reset-password") suspend fun resetPassword(@Body body: Map<String, String>): MessageResponse

    @GET("api/txns/dashboard") suspend fun dashboard(): DashboardStats
    @GET("api/txns/analysis") suspend fun analysis(): AnalysisStats
    @GET("api/txns") suspend fun listTxns(
        @Query("page") page: Int = 1, @Query("per_page") perPage: Int = 50,
        @Query("sort_by") sortBy: String = "date", @Query("sort_dir") sortDir: String = "desc",
        @Query("category") category: String? = null,
        @Query("from") from: String? = null, @Query("to") to: String? = null
    ): TxnListResponse
    @GET("api/txns/categories") suspend fun listCategories(): List<String>
    @POST("api/txns") suspend fun createTxn(@Body body: CreateTxnReq): TxnRow
    @PATCH("api/txns/{id}/transfer") suspend fun toggleTransfer(@Path("id") id: String, @Body body: ToggleTransferReq): MessageResponse
    @PATCH("api/txns/{id}/investment") suspend fun toggleInvestment(@Path("id") id: String, @Body body: ToggleInvestmentReq): MessageResponse
    @PATCH("api/txns/{id}/notes") suspend fun updateNotes(@Path("id") id: String, @Body body: UpdateNotesReq): MessageResponse

    @GET("api/accounts") suspend fun listAccounts(): List<UserAccount>
    @POST("api/accounts") suspend fun createAccount(@Body body: CreateAccountReq): UserAccount
    @DELETE("api/accounts/{id}") suspend fun deleteAccount(@Path("id") id: String): MessageResponse

    @GET("api/rules") suspend fun listRules(): List<CategoryRule>
    @POST("api/rules") suspend fun createRule(@Body body: CreateRuleReq): CategoryRule
    @DELETE("api/rules/{id}") suspend fun deleteRule(@Path("id") id: String): MessageResponse
    @POST("api/rules/apply") suspend fun applyRules(): MessageResponse

    @GET("api/budgets") suspend fun listBudgets(): List<Budget>
    @POST("api/budgets") suspend fun createBudget(@Body body: CreateBudgetReq): Budget
    @DELETE("api/budgets/{id}") suspend fun deleteBudget(@Path("id") id: String): MessageResponse
    @GET("api/budgets/status") suspend fun budgetStatus(): List<BudgetStatus>

    @GET("api/portfolio/snapshot") suspend fun portfolioSnapshot(): NetWorthSnapshot
    @POST("api/portfolio/assets") suspend fun createAsset(@Body body: Map<String, Any>): PortfolioAsset
    @DELETE("api/portfolio/assets/{id}") suspend fun deleteAsset(@Path("id") id: String): MessageResponse
    @POST("api/portfolio/liabilities") suspend fun createLiability(@Body body: Map<String, Any>): PortfolioLiability
    @DELETE("api/portfolio/liabilities/{id}") suspend fun deleteLiability(@Path("id") id: String): MessageResponse

    @Multipart @POST("api/ingest/upload") suspend fun uploadStatement(@Part file: MultipartBody.Part): Map<String, Any>
    @HTTP(method = "DELETE", path = "api/ingest/clear", hasBody = true) suspend fun clearAllData(@Body body: Map<String, Boolean> = mapOf("confirm" to true)): MessageResponse

    @GET("api/chat/history") suspend fun chatHistory(): List<ChatHistoryResponse>
    @POST("api/chat/ask") suspend fun chatAsk(@Body body: ChatAskRequest): ChatAskResponse
}
