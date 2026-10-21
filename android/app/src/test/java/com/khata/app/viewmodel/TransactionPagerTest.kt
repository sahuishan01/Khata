package com.khata.app.viewmodel

import com.khata.app.api.TxnListResponse
import com.khata.app.api.TxnRow
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.yield
import org.junit.Assert.*
import org.junit.Test

class TransactionPagerTest {
    private fun row(id: Int) = TxnRow(id = "$id", valueDate = "2026-10-15", description = "Payment $id",
        amount = 1.0, direction = "debit", category = "Uncategorized", bank = "HDFC", isTransfer = false, notes = "")
    private fun page(p: Int, total: Int = 5) = TxnListResponse((0 until total).drop((p - 1) * 2).take(2).map(::row), total, p, 2)

    @Test fun scrollingLoadsEveryPageAndStopsAtTheEnd() = runBlocking {
        val requested = mutableListOf<Int>()
        val pager = TransactionPager { p, _ -> requested += p; page(p) }
        pager.refresh(TxnFilter())
        repeat(4) { pager.loadMore() }
        assertEquals(listOf(1, 2, 3), requested)
        assertEquals(listOf("0", "1", "2", "3", "4"), pager.state.value.list!!.data.map { it.id })
        assertFalse(pager.state.value.hasMore)
    }

    @Test fun failedPageRetainsRowsAndRetryUsesTheSamePage() = runBlocking {
        val requested = mutableListOf<Int>()
        var fail = true
        val pager = TransactionPager { p, _ -> requested += p; if (p == 2 && fail) error("offline") else page(p) }
        pager.refresh(TxnFilter()); pager.loadMore()
        assertEquals(2, pager.state.value.list!!.data.size)
        assertNotNull(pager.state.value.moreError)
        fail = false; pager.loadMore()
        assertEquals(listOf(1, 2, 2), requested)
        assertEquals(4, pager.state.value.list!!.data.size)
        assertNull(pager.state.value.moreError)
    }

    @Test fun aFilterChangeRejectsAnOlderPageAndPreservesAllQueryFields() = runBlocking {
        val waiting = CompletableDeferred<TxnListResponse>()
        val filters = mutableListOf<TxnFilter>()
        val pager = TransactionPager { p, f -> filters += f; if (p == 2 && f.search.isEmpty()) waiting.await() else page(p) }
        pager.refresh(TxnFilter())
        val old = async { pager.loadMore() }; yield()
        val next = TxnFilter(sortBy = "amount", sortDir = "asc", category = "Food", from = "2026-01-01", to = "2026-12-31", search = "SHOP", duplicatesOnly = true)
        pager.refresh(next); waiting.complete(page(2)); old.await()
        assertEquals(1, pager.state.value.list!!.page)
        assertEquals(2, pager.state.value.list!!.data.size)
        pager.loadMore()
        assertEquals(next, filters.last())
    }

    @Test fun repeatedBottomCallbacksDoNotRequestTheSamePageTwice() = runBlocking {
        val waiting = CompletableDeferred<TxnListResponse>()
        var requests = 0
        val pager = TransactionPager { p, _ -> requests++; if (p == 2) waiting.await() else page(p) }
        pager.refresh(TxnFilter())
        val first = async { pager.loadMore() }; yield()
        pager.loadMore(); pager.loadMore()
        assertEquals(2, requests)
        waiting.complete(page(2)); first.await()
        assertEquals(4, pager.state.value.list!!.data.size)
    }

    @Test fun refreshRetainsLoadedDepthAndRemovesDuplicatesWithoutDiscardingRowsOnFailure() = runBlocking {
        var fail = false
        val pager = TransactionPager { p, _ -> if (fail) error("offline") else page(p) }
        pager.refresh(TxnFilter()); pager.loadMore(); pager.loadMore()
        pager.refresh(TxnFilter())
        assertEquals(5, pager.state.value.list!!.data.size)
        assertFalse(pager.state.value.hasMore)
        fail = true; pager.refresh(TxnFilter())
        assertEquals(5, pager.state.value.list!!.data.size)
        assertNotNull(pager.state.value.error)
    }
}
