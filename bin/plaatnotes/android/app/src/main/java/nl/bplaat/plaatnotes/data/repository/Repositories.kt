package nl.bplaat.plaatnotes.data.repository

import nl.bplaat.plaatnotes.core.network.ApiClient
import nl.bplaat.plaatnotes.data.models.*

sealed class ApiResult<out T> {
    data class Success<T>(val data: T) : ApiResult<T>()
    data class Error(val message: String) : ApiResult<Nothing>()
    data object Loading : ApiResult<Nothing>()
}

class AuthRepository {
    suspend fun login(email: String, password: String): ApiResult<LoginResponse> {
        return try {
            val response = ApiClient.api.login(email, password)
            ApiResult.Success(response)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Login failed")
        }
    }

    suspend fun logout(): ApiResult<Unit> {
        return try {
            ApiClient.api.logout()
            ApiClient.clearAuthToken()
            ApiResult.Success(Unit)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Logout failed")
        }
    }
}

class UserRepository {
    suspend fun getUser(userId: String): ApiResult<User> {
        return try {
            val user = ApiClient.api.getUser(userId)
            ApiResult.Success(user)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to fetch user")
        }
    }

    suspend fun updateUser(
        userId: String,
        firstName: String,
        lastName: String,
        email: String,
        role: UserRole
    ): ApiResult<User> {
        return try {
            val user = ApiClient.api.updateUser(
                userId,
                firstName,
                lastName,
                email,
                role.name.lowercase()
            )
            ApiResult.Success(user)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to update user")
        }
    }

    suspend fun changePassword(
        userId: String,
        oldPassword: String,
        newPassword: String
    ): ApiResult<Unit> {
        return try {
            ApiClient.api.changePassword(userId, oldPassword, newPassword)
            ApiResult.Success(Unit)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to change password")
        }
    }
}

class NoteRepository {
    suspend fun getNotes(
        query: String? = null,
        page: Int = 1,
        limit: Int = 20
    ): ApiResult<NoteIndexResponse> {
        return try {
            val response = ApiClient.api.getNotes(query, page, limit)
            ApiResult.Success(response)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to fetch notes")
        }
    }

    suspend fun createNote(body: String): ApiResult<Note> {
        return try {
            val note = ApiClient.api.createNote(body)
            ApiResult.Success(note)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to create note")
        }
    }

    suspend fun getNote(noteId: String): ApiResult<Note> {
        return try {
            val note = ApiClient.api.getNote(noteId)
            ApiResult.Success(note)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to fetch note")
        }
    }

    suspend fun updateNote(noteId: String, body: String): ApiResult<Note> {
        return try {
            val note = ApiClient.api.updateNote(noteId, body)
            ApiResult.Success(note)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to update note")
        }
    }

    suspend fun deleteNote(noteId: String): ApiResult<Unit> {
        return try {
            ApiClient.api.deleteNote(noteId)
            ApiResult.Success(Unit)
        } catch (e: Exception) {
            ApiResult.Error(e.message ?: "Failed to delete note")
        }
    }
}
