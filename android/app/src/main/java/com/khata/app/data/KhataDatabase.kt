package com.khata.app.data

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase

@Database(
    entities = [LocalTransaction::class, SyncState::class],
    version = 2,
    exportSchema = false,
)
abstract class KhataDatabase : RoomDatabase() {
    abstract fun transactionDao(): TransactionDao
    abstract fun syncStateDao(): SyncStateDao

    companion object {
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
                    .addMigrations(MIGRATION_1_2)
                    .build().also { INSTANCE = it }
            }
        }
    }
}
