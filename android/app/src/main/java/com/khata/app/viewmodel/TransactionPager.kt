package com.khata.app.viewmodel

import com.khata.app.api.TxnListResponse
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

data class TransactionPages(
    val list: TxnListResponse? = null,
    val loading: Boolean = false,
    val loadingMore: Boolean = false,
    val hasMore: Boolean = false,
    val error: String? = null,
    val moreError: String? = null,
)

/** One filter owns a sequence of pages; stale requests cannot append to a new filter. */
class TransactionPager(private val fetch: suspend (Int, TxnFilter) -> TxnListResponse) {
    private val mutable = MutableStateFlow(TransactionPages())
    val state = mutable.asStateFlow()
    private var generation = 0
    private var filter: TxnFilter? = null

    suspend fun refresh(next: TxnFilter) {
        val request = ++generation
        val previous = mutable.value
        val keep = filter == next
        val pages = if (keep) previous.list?.page ?: 1 else 1
        filter = next
        mutable.value = if (keep) previous.copy(loading = true, loadingMore = false, error = null, moreError = null)
            else TransactionPages(loading = true)
        try {
            var result: TxnListResponse? = null
            var remaining = false
            for (page in 1..pages) {
                val response = fetch(page, next)
                if (request != generation) return
                result = response.copy(data = ((result?.data ?: emptyList()) + response.data).distinctBy { it.id })
                remaining = more(response)
                if (!remaining) break
            }
            mutable.value = TransactionPages(list = result, hasMore = remaining)
        } catch (e: CancellationException) { throw e }
        catch (_: Exception) {
            if (request == generation) mutable.value = mutable.value.copy(loading = false, error = "Could not refresh transactions. Please retry.")
        }
    }

    suspend fun loadMore() {
        val before = mutable.value
        val current = before.list ?: return
        val query = filter ?: return
        if (before.loading || before.loadingMore || !before.hasMore) return
        val request = generation
        mutable.value = before.copy(loadingMore = true, moreError = null)
        try {
            val response = fetch(current.page + 1, query)
            if (request != generation) return
            mutable.value = mutable.value.copy(
                list = response.copy(data = (current.data + response.data).distinctBy { it.id }),
                loadingMore = false, hasMore = more(response), moreError = null)
        } catch (e: CancellationException) { throw e }
        catch (_: Exception) {
            if (request == generation) mutable.value = mutable.value.copy(loadingMore = false,
                moreError = "Could not load more transactions. Tap to retry.")
        }
    }

    private fun more(response: TxnListResponse) = response.data.isNotEmpty() &&
        response.page.toLong() * response.perPage < response.total
}
