import { authServiceClient } from '@service-auth/client';
import { action, useSubmission } from '@solidjs/router';
import { Stage } from './Shared';

// Initiates the password login flow (decoupled from passwordless verification code).
export const sendEmailCode = action(async (formData: FormData) => {
  const email = formData.get('email');
  if (!email || typeof email !== 'string')
    throw new Error('请输入有效的邮箱地址');

  const cleanEmail = email.trim();
  const password = formData.get('password');
  if (!password || typeof password !== 'string') return 'isPasswordLogin';

  const maybeTokens = await authServiceClient.passwordLogin({
    password,
    email: cleanEmail,
  });
  if (maybeTokens.isErr()) {
    const msg =
      maybeTokens.error?.message || '登录失败，请检查账号和密码后重试。';
    throw new Error(msg);
  }

  return 'LoggedIn';
}, 'password-login');

/// True when the email step succeeded and the verify step should show.
export function sentEmailCode(
  result: Awaited<ReturnType<typeof sendEmailCode>> | undefined
): boolean {
  return result === true || (typeof result === 'object' && !!result);
}

/// The local-backend auto-login code, when the email step returned one.
export function autoLoginCode(
  result: Awaited<ReturnType<typeof sendEmailCode>> | undefined
): string | undefined {
  if (typeof result === 'object' && result && 'autoCode' in result) {
    return result.autoCode;
  }
}

export function useResetEmailCode(setStage: (next: Stage) => void) {
  const submission = useSubmission(sendEmailCode);
  return () => {
    submission.clear();
    setStage(Stage.Email);
  };
}
