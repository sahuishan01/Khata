package com.khata.app.data

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

@Database(
    entities = [LocalTransaction::class, SyncState::class, com.khata.app.sms.SmsDecisionJob::class],
    version = 3,
    exportSchema = false,
)
abstract class KhataDatabase : RoomDatabase() {
    abstract fun transactionDao(): TransactionDao
    abstract fun syncStateDao(): SyncStateDao
    abstract fun smsDecisionDao(): com.khata.app.sms.SmsDecisionDao

    companion object {
        val MIGRATION_2_3 = object : Migration(2, 3) {
            override fun migrate(db: SupportSQLiteDatabase) {
                db.execSQL("CREATE TABLE IF NOT EXISTS sms_decision_jobs (id TEXT NOT NULL PRIMARY KEY, owner TEXT NOT NULL, sender TEXT NOT NULL, body TEXT, receivedAt INTEGER NOT NULL, queuedAt INTEGER NOT NULL, status TEXT NOT NULL, reason TEXT NOT NULL, legacyReceivedAt INTEGER)")
                db.execSQL("CREATE INDEX IF NOT EXISTS index_sms_decision_jobs_owner_status ON sms_decision_jobs(owner,status)")
            }
        }
        val MIGRATION_1_2 = object : Migration(1, 2) {
            override fun migrate(db: SupportSQLiteDatabase) {
                db.execSQL("ALTER TABLE transactions ADD COLUMN smsSource TEXT")
            }
        }
        @Volatile
        private var INSTANCE: KhataDatabase? = null

        fun getInstance(context: Context): KhataDatabase {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: Room.databaseBuilder(context, KhataDatabase::class.java, "khata.db")
                    .addMigrations(MIGRATION_1_2, MIGRATION_2_3)
                    .build().also { INSTANCE = it }
            }
        }
    }
}
