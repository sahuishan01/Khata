package com.khata.app.api

import com.khata.app.BuildConfig
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.HttpUrl.Companion.toHttpUrl
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.util.concurrent.TimeUnit
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object NetworkModule {

    @Provides
    @Singleton
    fun provideTokenInterceptor(tokenManager: TokenManager): Interceptor {
        return Interceptor { chain ->
            val request = chain.request().newBuilder()
            tokenManager.getTokenSync()?.let { token ->
                request.addHeader("Authorization", "Bearer $token")
            }
            chain.proceed(request.build())
        }
    }

    @Provides
    @Singleton
    fun provideServerUrlInterceptor(tokenManager: TokenManager): Interceptor {
        val defaultBase = BuildConfig.API_BASE_URL.toHttpUrl()
        return Interceptor { chain ->
            val serverUrl = tokenManager.getServerUrlSync()
            val original = chain.request()
            val newUrl = if (serverUrl != BuildConfig.API_BASE_URL) {
                val customBase = serverUrl.toHttpUrl()
                original.url.newBuilder()
                    .scheme(customBase.scheme)
                    .host(customBase.host)
                    .port(customBase.port)
                    .build()
            } else {
                original.url
            }
            chain.proceed(original.newBuilder().url(newUrl).build())
        }
    }

    @Provides
    @Singleton
    fun provideOkHttpClient(
        tokenInterceptor: Interceptor,
        serverUrlInterceptor: Interceptor
    ): OkHttpClient {
        val builder = OkHttpClient.Builder()
            .addInterceptor(serverUrlInterceptor)
            .addInterceptor(tokenInterceptor)
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)

        if (BuildConfig.DEBUG) {
            val logging = HttpLoggingInterceptor().apply {
                level = HttpLoggingInterceptor.Level.BODY
                redactHeader("Authorization")
            }
            builder.addInterceptor(logging)
        }

        return builder.build()
    }

    @Provides
    @Singleton
    fun provideRetrofit(client: OkHttpClient): Retrofit {
        return Retrofit.Builder()
            .baseUrl(BuildConfig.API_BASE_URL + "/")
            .client(client)
            .addConverterFactory(GsonConverterFactory.create())
            .build()
    }

    @Provides
    @Singleton
    fun provideKhataApi(retrofit: Retrofit): KhataApi {
        return retrofit.create(KhataApi::class.java)
    }
}
