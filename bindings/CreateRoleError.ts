// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { GeneralError } from "./GeneralError";

export type CreateRoleError = { is: "General", data: GeneralError } | { is: "DoesNotExist", data: string };