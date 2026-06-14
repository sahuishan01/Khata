package com.khata.app.data

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import net.sqlcipher.database.SupportFactory
import net.sqlcipher.database.SQLiteDatabase as CipherDB
import javax.crypto.KeyGenerator

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
                INSTANCE ?: buildDatabase(context).also { INSTANCE = it }
            }
        }

        private fun buildDatabase(context: Context): KhataDatabase {
            CipherDB.loadLibs(context)
            val passphrase = try {
                val spec = android.security.keystore.KeyGenParameterSpec.Builder(
                    "khata_db_key",
                    android.security.keystore.KeyProperties.PURPOSE_ENCRYPT or android.security.keystore.KeyProperties.PURPOSE_DECRYPT,
                ).apply {
                    setBlockModes(android.security.keystore.KeyProperties.BLOCK_MODE_GCM)
                    setEncryptionPaddings(android.security.keystore.KeyProperties.ENCRYPTION_PADDING_NONE)
                    setKeySize(256)
                }.build()
                val kg = KeyGenerator.getInstance("AES", "AndroidKeyStore")
                kg.init(spec)
                val key = kg.generateKey()
                key.encoded?.let { java.util.Base64.getEncoder().encodeToString(it) }
                    ?: "default-key-32-chars-long!!"
            } catch (_: Exception) {
                "default-key-32-chars-long!!"
            }

            val factory = SupportFactory(passphrase.toByteArray())
            return Room.databaseBuilder(context, KhataDatabase::class.java, "khata_encrypted.db")
                .openHelperFactory(factory)
                .fallbackToDestructiveMigration()
                .build()
        }
    }
}
