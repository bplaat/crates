package nl.bplaat.plaatnotes.data.api

import nl.bplaat.plaatnotes.data.models.*
import retrofit2.http.*

interface PlaatNotesApi {
    @POST("auth/login")
    @FormUrlEncoded
    suspend fun login(
        @Field("email") email: String,
        @Field("password") password: String
    ): LoginResponse

    @POST("auth/logout")
    suspend fun logout()

    @GET("users/{id}")
    suspend fun getUser(@Path("id") userId: String): User

    @PUT("users/{id}")
    @FormUrlEncoded
    suspend fun updateUser(
        @Path("id") userId: String,
        @Field("firstName") firstName: String,
        @Field("lastName") lastName: String,
        @Field("email") email: String,
        @Field("role") role: String
    ): User

    @POST("users/{id}/change-password")
    @FormUrlEncoded
    suspend fun changePassword(
        @Path("id") userId: String,
        @Field("oldPassword") oldPassword: String,
        @Field("newPassword") newPassword: String
    )

    @GET("notes")
    suspend fun getNotes(
        @Query("q") query: String? = null,
        @Query("page") page: Int = 1,
        @Query("limit") limit: Int = 20
    ): NoteIndexResponse

    @POST("notes")
    @FormUrlEncoded
    suspend fun createNote(@Field("body") body: String): Note

    @GET("notes/{id}")
    suspend fun getNote(@Path("id") noteId: String): Note

    @PUT("notes/{id}")
    @FormUrlEncoded
    suspend fun updateNote(
        @Path("id") noteId: String,
        @Field("body") body: String
    ): Note

    @DELETE("notes/{id}")
    suspend fun deleteNote(@Path("id") noteId: String)
}
