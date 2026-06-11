package com.khata.app.api

import okhttp3.MultipartBody
import retrofit2.http.*

interface KhataApi {

    // Auth
    @POST("api/auth/login")
    suspend fun login(@Body body: Map<String, String>): AuthResponse

    @POST("api/auth/setup")
    suspend fun setup(@Body body: Map<String, String>): AuthResponse

    @GET("api/auth/setup-status")
    suspend fun setupStatus(): SetupStatusResponse

    @GET("api/auth/me")
    suspend fun me(): MeResponse

    @GET("api/auth/users")
    suspend fun listUsers(): List<UserResponse>

    @POST("api/auth/users")
    suspend fun createUser(@Body body: Map<String, String>): UserResponse

    @DELETE("api/auth/users/{id}")
    suspend fun deleteUser(@Path("id") id: String): MessageResponse

    @POST("api/auth/reset-password")
    suspend fun resetPassword(@Body body: Map<String, String>): MessageResponse

    // Dashboard
    @GET("api/txns/dashboard")
    suspend fun dashboard(): DashboardStats

    @GET("api/txns/analysis")
    suspend fun analysis(): AnalysisStats

    // Transactions
    @GET("api/txns")
    suspend fun listTxns(
        @Query("page") page: Int = 1,
        @Query("per_page") perPage: Int = 50,
        @Query("sort_by") sortBy: String = "date",
        @Query("sort_dir") sortDir: String = "desc",
        @Query("category") category: String? = null
    ): TxnListResponse

    @GET("api/txns/categories")
    suspend fun listCategories(): List<String>

    // Upload
    @Multipart
    @POST("api/ingest/upload")
    suspend fun uploadStatement(@Part file: MultipartBody.Part): Map<String, Any>

    @HTTP(method = "DELETE", path = "api/ingest/clear", hasBody = true)
    suspend fun clearAllData(@Body body: Map<String, Boolean> = mapOf("confirm" to true)): MessageResponse

    // Chat
    @GET("api/chat/history")
    suspend fun chatHistory(): List<ChatHistoryResponse>

    @POST("api/chat/ask")
    suspend fun chatAsk(@Body body: ChatAskRequest): ChatAskResponse
}
