package com.khata.app.sms

import androidx.room.*
import kotlinx.coroutines.flow.Flow

@Entity(tableName = "sms_decision_jobs", indices = [Index(value = ["owner", "status"])])
data class SmsDecisionJob(
    @PrimaryKey val id: String,
    val owner: String,
    val sender: String,
    val body: String?,
    val receivedAt: Long,
    val queuedAt: Long,
    val status: String = "pending",
    val reason: String = "",
)

@Dao
interface SmsDecisionDao {
    @Insert(onConflict = OnConflictStrategy.IGNORE) suspend fun insert(job: SmsDecisionJob): Long
    @Query("SELECT * FROM sms_decision_jobs WHERE owner=:owner AND status='pending' ORDER BY queuedAt LIMIT 12")
    suspend fun pending(owner: String): List<SmsDecisionJob>
    @Query("SELECT * FROM sms_decision_jobs WHERE owner=:owner AND status='review' ORDER BY queuedAt DESC LIMIT 50")
    suspend fun review(owner: String): List<SmsDecisionJob>
    // One pending marker plus recent reviews keeps both states visible during long scans.
    @Query("SELECT * FROM sms_decision_jobs WHERE owner=:owner AND (id=(SELECT id FROM sms_decision_jobs WHERE owner=:owner AND status='pending' ORDER BY queuedAt LIMIT 1) OR status='review') ORDER BY status='pending' DESC, queuedAt DESC LIMIT 51")
    fun active(owner: String): Flow<List<SmsDecisionJob>>
    @Query("UPDATE sms_decision_jobs SET status=:status, reason=:reason, body=CASE WHEN :retain THEN body ELSE NULL END, sender=CASE WHEN :retain THEN sender ELSE '' END WHERE id=:id")
    suspend fun finish(id: String, status: String, reason: String = "", retain: Boolean = false)
    @Query("UPDATE sms_decision_jobs SET status='pending', reason='' WHERE owner=:owner AND status='review'")
    suspend fun retryReview(owner: String)
    @Query("DELETE FROM sms_decision_jobs") suspend fun clearAll()
}
