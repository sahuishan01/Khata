package com.khata.app.api

import com.google.gson.annotations.SerializedName

data class AuthResponse(
    val token: String,
    @SerializedName("must_reset_password") val mustResetPassword: Boolean = false
)
data class MeResponse(val id: String, val email: String, val role: String)
data class SetupStatusResponse(@SerializedName("setup_required") val setupRequired: Boolean)
data class UserResponse(val id: String, val email: String, val role: String)

data class DashboardStats(
    @SerializedName("total_spent") val totalSpent: Double,
    @SerializedName("total_earned") val totalEarned: Double,
    @SerializedName("total_invested") val totalInvested: Double,
    val net: Double,
    val monthly: List<MonthBucket>,
    @SerializedName("top_debits") val topDebits: List<TopMerchant>
)

data class MonthBucket(val month: String, val spent: Double, val earned: Double)
data class TopMerchant(val description: String, val total: Double)

data class AnalysisStats(
    @SerializedName("category_breakdown") val categoryBreakdown: List<CategoryBucket>,
    @SerializedName("savings_rate_pct") val savingsRatePct: Double,
    @SerializedName("avg_daily_spend") val avgDailySpend: Double,
    @SerializedName("month_comparison") val monthComparison: MonthComparison,
    @SerializedName("largest_expense") val largestExpense: LargestExpense?,
    @SerializedName("total_transactions") val totalTransactions: Int,
    @SerializedName("total_invested") val totalInvested: Double
)

data class CategoryBucket(val category: String, val amount: Double, @SerializedName("txn_count") val txnCount: Int, val pct: Double)
data class MonthComparison(@SerializedName("this_month") val thisMonth: Double, @SerializedName("last_month") val lastMonth: Double, @SerializedName("change_pct") val changePct: Double)
data class LargestExpense(val description: String, val amount: Double, @SerializedName("value_date") val valueDate: String)

data class TxnListResponse(val data: List<TxnRow>, val total: Int, val page: Int, @SerializedName("per_page") val perPage: Int)
data class TxnRow(
    val id: String, @SerializedName("value_date") val valueDate: String,
    val description: String, val amount: Double, val direction: String,
    val category: String, val bank: String,
    @SerializedName("is_transfer") val isTransfer: Boolean,
    val notes: String,
    @SerializedName("client_id") val clientId: String? = null,
    val rev: Long = 0,
    @SerializedName("base_rev") val baseRev: Long = 0,
    val deleted: Boolean = false,
)

data class ChatHistoryResponse(val id: String, val role: String, val content: String, @SerializedName("sql_used") val sqlUsed: String?)
data class ChatAskRequest(val question: String)
data class ChatAskResponse(val answer: String, @SerializedName("sql_used") val sqlUsed: String)
data class MessageResponse(val message: String)

// Categories
data class Category(val id: String, val name: String, @SerializedName("txn_type") val txnType: String, val color: String, val description: String)
data class CreateCategoryReq(val name: String, @SerializedName("txn_type") val txnType: String, val color: String? = null, val description: String? = null)

// Accounts
data class UserAccount(val id: String, val label: String, val identifier: String)
data class CreateAccountReq(val label: String, val identifier: String)

// Rules
data class CategoryRule(val id: String, val pattern: String, val category: String)
data class CreateRuleReq(val pattern: String, val category: String)

// Budgets
data class Budget(val id: String, val category: String, @SerializedName("monthly_limit") val monthlyLimit: Double)
data class BudgetStatus(val category: String, @SerializedName("monthly_limit") val monthlyLimit: Double, val spent: Double, val pct: Double)
data class CreateBudgetReq(val category: String, @SerializedName("monthly_limit") val monthlyLimit: Double)

// Portfolio
data class PortfolioAsset(val id: String, val name: String, @SerializedName("asset_type") val assetType: String, val value: Double)
data class PortfolioLiability(val id: String, val name: String, @SerializedName("liability_type") val liabilityType: String, val value: Double)
data class NetWorthSnapshot(
    @SerializedName("total_assets") val totalAssets: Double,
    @SerializedName("total_liabilities") val totalLiabilities: Double,
    @SerializedName("net_worth") val netWorth: Double,
    val assets: List<PortfolioAsset>,
    val liabilities: List<PortfolioLiability>
)

// Manual transaction
data class CreateTxnReq(
    @SerializedName("txn_date") val txnDate: String,
    @SerializedName("value_date") val valueDate: String,
    val description: String, val amount: Double, val direction: String,
    val category: String, @SerializedName("bank_ref") val bankRef: String?,
    val notes: String?
)

// Transfer / Investment toggles
data class ToggleTransferReq(@SerializedName("is_transfer") val isTransfer: Boolean)
data class UpdateNotesReq(val notes: String)

// Sync models
data class SyncPullResponse(
    val txns: List<TxnRow>,
    @SerializedName("new_rev") val newRev: Long,
    @SerializedName("has_more") val hasMore: Boolean,
)

data class SyncPushOp(
    @SerializedName("client_id") val clientId: String,
    @SerializedName("base_rev") val baseRev: Long,
    val amount: Double? = null,
    val direction: String? = null,
    val description: String? = null,
    val category: String? = null,
    @SerializedName("txn_date") val txnDate: String? = null,
    @SerializedName("value_date") val valueDate: String? = null,
    val notes: String? = null,
    @SerializedName("is_transfer") val isTransfer: Boolean = false,
    val deleted: Boolean = false,
)

data class SyncPushResult(
    val accepted: List<SyncAccepted>,
    val conflicts: List<SyncConflictDetail>,
)

data class SyncAccepted(
    @SerializedName("client_id") val clientId: String,
    @SerializedName("server_rev") val serverRev: Long,
)

data class SyncConflictDetail(
    @SerializedName("client_id") val clientId: String,
    @SerializedName("base_rev") val baseRev: Long,
    @SerializedName("server_rev") val serverRev: Long,
    @SerializedName("server_txn") val serverTxn: TxnRow,
    @SerializedName("local_txn") val localTxn: Any? = null,
)

fun TxnRow.toLocal() = com.khata.app.data.LocalTransaction(
    clientId = this.clientId?.toString() ?: "",
    description = this.description,
    amount = this.amount,
    direction = this.direction,
    category = this.category,
    valueDate = this.valueDate.toString(),
    rev = this.rev,
    baseRev = this.baseRev,
    deleted = this.deleted,
)
