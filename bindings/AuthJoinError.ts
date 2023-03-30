// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { GeneralError } from "./GeneralError";

export type AuthJoinError = { is: "InvaildNameChars" } | { is: "BadNameLength" } | { is: "TooShortPassword" } | { is: "TooLongPassword" } | { is: "NameTaken" } | { is: "EmailTaken" } | { is: "General", data: GeneralError };