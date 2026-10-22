import type { TFunction } from "i18next";

/** 后端错误的返回形状：`code` 为稳定错误码，`detail` 为可选补充信息。 */
export interface BackendError {
  code: string;
  detail?: string;
}

/**
 * 提取后端错误信息：优先用可读的 detail，其次才回退到错误码。
 *
 * 后端只返回稳定错误码与可选 detail，不返回堆栈；直接展示错误码对用户无意义，
 * 因此有 detail 时只展示 detail。
 */
export function describeError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "code" in error) {
    const detail = (error as BackendError).detail;
    return detail ? String(detail) : String((error as BackendError).code);
  }
  return String(error);
}

/**
 * 扫描警告码格式化：优先用本地化文案，否则回退原始警告码。
 *
 * 警告码（`Tsf/*`）在本地化资源中同名存在，查不到时说明资源缺失，
 * 此时展示原始警告码比展示键名更容易定位问题。
 */
export function formatWarning(
  t: TFunction,
  code: string,
  args: string[] | null,
): string {
  const values = args ?? [];
  const options: Record<string, unknown> = {};
  values.forEach((value, index) => {
    options[index] = value;
  });
  const translated = t(code, options);
  return translated === code ? code : translated;
}
