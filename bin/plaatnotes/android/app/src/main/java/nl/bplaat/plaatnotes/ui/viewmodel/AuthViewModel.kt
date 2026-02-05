package nl.bplaat.plaatnotes.ui.viewmodel

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import nl.bplaat.plaatnotes.core.network.ApiClient
import nl.bplaat.plaatnotes.core.storage.TokenStorage
import nl.bplaat.plaatnotes.data.repository.ApiResult
import nl.bplaat.plaatnotes.data.repository.AuthRepository

data class LoginUiState(
    val isLoading: Boolean = false,
    val error: String? = null,
    val userId: String? = null,
    val token: String? = null
)

class AuthViewModel(private val application: Application) : AndroidViewModel(application) {
    private val authRepository = AuthRepository()
    private val _loginState = MutableStateFlow(LoginUiState())
    val loginState: StateFlow<LoginUiState> = _loginState.asStateFlow()

    fun login(email: String, password: String) {
        viewModelScope.launch {
            _loginState.value = _loginState.value.copy(isLoading = true, error = null)
            val result = authRepository.login(email, password)
            when (result) {
                is ApiResult.Success -> {
                    ApiClient.setAuthToken(result.data.token)
                    TokenStorage.saveToken(application, result.data.token, result.data.userId)
                    _loginState.value = _loginState.value.copy(
                        isLoading = false,
                        userId = result.data.userId,
                        token = result.data.token
                    )
                }
                is ApiResult.Error -> {
                    _loginState.value = _loginState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun logout() {
        viewModelScope.launch {
            authRepository.logout()
            TokenStorage.clearToken(application)
            resetLoginState()
        }
    }

    fun resetLoginState() {
        _loginState.value = LoginUiState()
    }
}
