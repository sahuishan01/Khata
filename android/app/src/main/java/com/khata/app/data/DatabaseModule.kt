package com.khata.app.data

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {

    @Provides
    @Singleton
    fun provideKhataDatabase(@ApplicationContext context: Context): KhataDatabase {
        return KhataDatabase.getInstance(context)
    }

    @Provides
    @Singleton
    fun provideTransactionDao(db: KhataDatabase): TransactionDao {
        return db.transactionDao()
    }

    @Provides
    @Singleton
    fun provideSyncStateDao(db: KhataDatabase): SyncStateDao {
        return db.syncStateDao()
    }
}
