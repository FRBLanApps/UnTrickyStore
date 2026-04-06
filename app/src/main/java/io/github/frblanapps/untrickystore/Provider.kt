/*
 * UnTrickyStore - ContentProvider for VBMeta hash
 *
 * Reads the vbmeta partition, computes SHA-256 digest, and returns
 * it via a ContentProvider `call()` method so the Rust binary can
 * query it from the shell via `content call --uri content://Provider --method GET`.
 *
 * Copyright (C) 2025-2026 FRBLanApps
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */
package io.github.frblanapps.untrickystore

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.net.Uri
import android.os.Bundle
import android.util.Log
import java.io.File
import java.io.RandomAccessFile
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.MessageDigest

class Provider : ContentProvider() {

    companion object {
        private const val TAG = "[UTS]"
        private const val VBMETA_MAGIC = "AVB0"
        private const val AVB_HEADER_SIZE = 256
    }

    override fun onCreate(): Boolean = true

    override fun call(method: String, arg: String?, extras: Bundle?): Bundle {
        val result = Bundle()
        try {
            val hash = computeVBMetaHash()
            Log.i(TAG, "VBHash=$hash")
            result.putString("result", "${hash}=VBHash")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to compute VBMeta hash", e)
            result.putString("result", "ERROR=${e.message}")
        }
        return result
    }

    private fun computeVBMetaHash(): String {
        val slot = getActiveSlot()
        val devPath = "/dev/block/by-name/vbmeta$slot"
        val dev = File(devPath)
        if (!dev.exists()) {
            throw IllegalStateException("vbmeta device not found: $devPath")
        }

        RandomAccessFile(dev, "r").use { raf ->
            // Read AVB header (first 256 bytes)
            val header = ByteArray(AVB_HEADER_SIZE)
            raf.readFully(header)

            // Verify magic
            val magic = String(header, 0, 4)
            if (magic != VBMETA_MAGIC) {
                throw IllegalStateException("Invalid vbmeta magic: $magic")
            }

            // Parse header to get total size of signed data
            // AVB header layout (from libavb):
            //   0-3:   magic "AVB0"
            //   4-7:   required_libavb_version_major
            //   8-11:  required_libavb_version_minor
            //   12-19: authentication_data_block_size
            //   20-27: auxiliary_data_block_size
            val buf = ByteBuffer.wrap(header).order(ByteOrder.BIG_ENDIAN)
            buf.position(12)
            val authBlockSize = buf.getLong()
            val auxBlockSize = buf.getLong()

            // Total data to hash: header + auth block + aux block
            val totalSize = AVB_HEADER_SIZE + authBlockSize + auxBlockSize
            raf.seek(0)
            val data = ByteArray(totalSize.toInt())
            raf.readFully(data)

            val digest = MessageDigest.getInstance("SHA-256")
            val hash = digest.digest(data)
            return hash.joinToString("") { "%02x".format(it) }
        }
    }

    private fun getActiveSlot(): String {
        return try {
            val process = Runtime.getRuntime().exec(arrayOf("getprop", "ro.boot.slot_suffix"))
            process.inputStream.bufferedReader().readLine()?.trim() ?: ""
        } catch (_: Exception) {
            ""
        }
    }

    // Required ContentProvider stubs
    override fun query(u: Uri, p: Array<String>?, s: String?, a: Array<String>?, o: String?): Cursor? = null
    override fun getType(uri: Uri): String? = null
    override fun insert(uri: Uri, values: ContentValues?): Uri? = null
    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<String>?): Int = 0
    override fun update(uri: Uri, values: ContentValues?, selection: String?, selectionArgs: Array<String>?): Int = 0
}
