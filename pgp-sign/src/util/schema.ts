export type SigningAlgorithm = "ed25519";

export interface PgpCardInfo {
    manufacturer: string,
    serialNumber: string,
    aid: string,
    signingAlgo: SigningAlgorithm,
    pubkeyBytes: Array<number>,
}

export interface ListCardsData {
    ListCards: PgpCardInfo[],
}

export interface SignMessageData {
    SignMessage: Array<number>,
}

export interface BlossError {
    type: string,
    details: any,
}
