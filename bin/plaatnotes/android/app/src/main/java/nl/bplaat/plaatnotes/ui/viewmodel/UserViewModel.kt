package nl.bplaat.plaatnotes.ui.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import nl.bplaat.plaatnotes.data.models.User
import nl.bplaat.plaatnotes.data.models.UserRole
import nl.bplaat.plaatnotes.data.repository.ApiResult
import nl.bplaat.plaatnotes.data.repository.UserRepository

data class UserUiState(
    val user: User? = null,
    val isLoading: Boolean = false,
    val error: String? = null,
    val updateSuccess: Boolean = false,
    val passwordChangeSuccess: Boolean = false
)

class UserViewModel : ViewModel() {
    private val userRepository = UserRepository()
    private val _userState = MutableStateFlow(UserUiState())
    val userState: StateFlow<UserUiState> = _userState.asStateFlow()

    fun loadUser(userId: String) {
        viewModelScope.launch {
            _userState.value = _userState.value.copy(isLoading = true, error = null)
            val result = userRepository.getUser(userId)
            when (result) {
                is ApiResult.Success -> {
                    _userState.value = _userState.value.copy(
                        isLoading = false,
                        user = result.data
                    )
                }
                is ApiResult.Error -> {
                    _userState.value = _userState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun updateUser(
        userId: String,
        firstName: String,
        lastName: String,
        email: String,
        role: UserRole
    ) {
        viewModelScope.launch {
            _userState.value = _userState.value.copy(isLoading = true, error = null)
            val result = userRepository.updateUser(userId, firstName, lastName, email, role)
            when (result) {
                is ApiResult.Success -> {
                    _userState.value = _userState.value.copy(
                        isLoading = false,
                        user = result.data,
                        updateSuccess = true
                    )
                }
                is ApiResult.Error -> {
                    _userState.value = _userState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun changePassword(userId: String, oldPassword: String, newPassword: String) {
        viewModelScope.launch {
            _userState.value = _userState.value.copy(isLoading = true, error = null)
            val result = userRepository.changePassword(userId, oldPassword, newPassword)
            when (result) {
                is ApiResult.Success -> {
                    _userState.value = _userState.value.copy(
                        isLoading = false,
                        passwordChangeSuccess = true
                    )
                }
                is ApiResult.Error -> {
                    _userState.value = _userState.value.copy(
                        isLoading = false,
                        error = result.message
                    )
                }
                is ApiResult.Loading -> {}
            }
        }
    }

    fun clearMessages() {
        _userState.value = _userState.value.copy(
            updateSuccess = false,
            passwordChangeSuccess = false,
            error = null
        )
    }
}
