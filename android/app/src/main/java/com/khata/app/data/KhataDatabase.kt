package com.khata.app.data

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase

@Database(
    entities = [LocalTransaction::class, SyncState::class],
    version = 1,
    exportSchema = false,
)
abstract class KhataDatabase : RoomDatabase() {
    abstract fun transactionDao(): TransactionDao
    abstract fun syncStateDao(): SyncStateDao

    companion object {
        @Volatile
        private var INSTANCE: KhataDatabase? = null

        fun getInstance(context: Context): KhataDatabase {
            return INSTANCE ?: synchronized(this) {
                INSTANCE ?: Room.databaseBuilder(context, KhataDatabase::class.java, "khata.db")
                    .fallbackToDestructiveMigration()
                    .build().also { INSTANCE = it }
            }
        }
    }
}
