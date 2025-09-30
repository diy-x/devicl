// Authentication related functions

// Check authentication status on page load
async function checkAuthStatus() {
    try {
        const response = await fetch('/api/auth/status');
        const data = await response.json();

        if (!data.authenticated) {
            window.location.href = '/login';
        }
    } catch (error) {
        console.error('Failed to check auth status:', error);
        window.location.href = '/login';
    }
}

// Logout function
async function logout() {
    try {
        const response = await fetch('/api/auth/logout', {
            method: 'POST'
        });

        if (response.ok) {
            window.location.href = '/login';
        } else {
            showNotification(getMessage('request_failed'), 'error');
        }
    } catch (error) {
        console.error('Logout error:', error);
        showNotification(getMessage('request_failed'), 'error');
    }
}

// Show change password modal
function showChangePasswordModal() {
    const modal = document.getElementById('changePasswordModal');
    modal.showModal();

    // Clear form
    document.getElementById('changePasswordForm').reset();
    const messageDiv = document.getElementById('passwordMessage');
    messageDiv.style.display = 'none';

    // Focus on old password field
    document.getElementById('oldPassword').focus();
}

// Close change password modal
function closeChangePasswordModal() {
    const modal = document.getElementById('changePasswordModal');
    modal.close();
}

// Handle change password form
document.addEventListener('DOMContentLoaded', function() {
    // Check authentication status
    checkAuthStatus();

    // Modal close button handler
    const closeModalBtn = document.getElementById('closeModal');
    if (closeModalBtn) {
        closeModalBtn.addEventListener('click', closeChangePasswordModal);
    }

    // Change password form handler
    const changePasswordForm = document.getElementById('changePasswordForm');
    if (changePasswordForm) {
        changePasswordForm.addEventListener('submit', async function(e) {
            e.preventDefault();

            const formData = new FormData(this);
            const oldPassword = formData.get('old_password');
            const newPassword = formData.get('new_password');
            const confirmPassword = formData.get('confirm_password');
            const messageDiv = document.getElementById('passwordMessage');

            // Client-side validation
            if (newPassword !== confirmPassword) {
                messageDiv.className = 'error-message';
                messageDiv.textContent = getMessage('password_mismatch');
                messageDiv.style.display = 'block';
                return;
            }

            if (newPassword.length < 6) {
                messageDiv.className = 'error-message';
                messageDiv.textContent = getMessage('password_too_short');
                messageDiv.style.display = 'block';
                return;
            }

            // Clear any previous messages
            messageDiv.style.display = 'none';

            try {
                const response = await fetch('/api/auth/change-password', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        old_password: oldPassword,
                        new_password: newPassword
                    })
                });

                const result = await response.json();

                if (response.ok && result.success) {
                    messageDiv.className = 'success-message';
                    messageDiv.textContent = getMessage('password_changed_success');
                    messageDiv.style.display = 'block';

                    // Close modal after a short delay
                    setTimeout(() => {
                        closeChangePasswordModal();
                        showNotification(getMessage('password_changed_success'), 'success');
                    }, 1500);
                } else {
                    messageDiv.className = 'error-message';
                    messageDiv.textContent = getMessage('password_change_error');
                    messageDiv.style.display = 'block';
                }
            } catch (error) {
                messageDiv.className = 'error-message';
                messageDiv.textContent = getMessage('request_failed') + ': ' + error.message;
                messageDiv.style.display = 'block';
            }
        });
    }

    // Clear password message when user types
    const passwordInputs = ['oldPassword', 'newPassword', 'confirmPassword'];
    passwordInputs.forEach(id => {
        const input = document.getElementById(id);
        if (input) {
            input.addEventListener('input', function() {
                const messageDiv = document.getElementById('passwordMessage');
                messageDiv.style.display = 'none';
            });
        }
    });
});