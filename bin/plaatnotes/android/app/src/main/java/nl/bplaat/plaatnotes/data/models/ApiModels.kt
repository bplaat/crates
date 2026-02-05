package nl.bplaat.plaatnotes.data.models

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class LoginRequest(
    val email: String,
    val password: String
)

@Serializable
data class LoginResponse(
    val userId: String,
    val token: String
)

@Serializable
data class User(
    val id: String,
    val firstName: String,
    val lastName: String,
    val email: String,
    val role: UserRole,
    val createdAt: String,
    val updatedAt: String
)

@Serializable
enum class UserRole {
    @SerialName("normal")
    NORMAL,
    @SerialName("admin")
    ADMIN
}

@Serializable
data class UserUpdateRequest(
    val firstName: String,
    val lastName: String,
    val email: String,
    val role: UserRole
)

@Serializable
data class UserChangePasswordRequest(
    val oldPassword: String,
    val newPassword: String
)

@Serializable
data class UserIndexResponse(
    val pagination: Pagination,
    val data: List<User>
)

@Serializable
data class Note(
    val id: String,
    val userId: String,
    val body: String,
    val createdAt: String,
    val updatedAt: String
)

@Serializable
data class NoteCreateRequest(
    val body: String
)

@Serializable
data class NoteUpdateRequest(
    val body: String
)

@Serializable
data class NoteIndexResponse(
    val pagination: Pagination,
    val data: List<Note>
)

@Serializable
data class Pagination(
    val page: Int,
    val limit: Int,
    val total: Int
)

@Serializable
data class ApiError(
    @SerialName("errors")
    val errorMap: Map<String, List<String>> = emptyMap()
) {
    fun getMessage(): String = errorMap.values.flatten().firstOrNull() ?: "Unknown error"
}
