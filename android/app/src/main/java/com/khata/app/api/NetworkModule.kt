package com.khata.app.api

import com.khata.app.BuildConfig
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.components.SingletonComponent
import okhttp3.Interceptor
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import retrofit2.converter.gson.GsonConverterFactory
import java.util.concurrent.TimeUnit
import javax.inject.Named
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object NetworkModule {

    @Provides
    @Singleton
    @Named("token")
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
    @Named("server")
    fun provideServerUrlInterceptor(tokenManager: TokenManager): Interceptor {
        return Interceptor { chain ->
            val serverUrl = tokenManager.getServerUrlSync()
            val original = chain.request()
            if (serverUrl != BuildConfig.API_BASE_URL) {
                try {
                    val custom = java.net.URL(serverUrl)
                    val port = if (custom.port == -1) {
                        if (custom.protocol == "https") 443 else 80
                    } else {
                        custom.port
                    }
                    val newUrl = original.url.newBuilder()
                        .scheme(custom.protocol)
                        .host(custom.host)
                        .port(port)
                        .build()
                    chain.proceed(original.newBuilder().url(newUrl).build())
                } catch (_: Exception) {
                    chain.proceed(original)
                }
            } else {
                chain.proceed(original)
            }
        }
    }

    @Provides
    @Singleton
    fun provideOkHttpClient(
        @Named("token") tokenInterceptor: Interceptor,
        @Named("server") serverUrlInterceptor: Interceptor
    ): OkHttpClient {
        val logging = HttpLoggingInterceptor().apply {
            redactHeader("Authorization")
            redactHeader("Cookie")
            level = if (BuildConfig.DEBUG) {
                HttpLoggingInterceptor.Level.BODY
            } else {
                HttpLoggingInterceptor.Level.NONE
            }
        }
        return OkHttpClient.Builder()
            .addInterceptor(serverUrlInterceptor)
            .addInterceptor(tokenInterceptor)
            .addInterceptor(logging)
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)
            .build()
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
