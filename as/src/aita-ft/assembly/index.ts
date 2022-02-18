import { context, storage, PersistentMap } from 'near-sdk-as';
import { FTContractMetadata } from './metadata';

const balances = new PersistentMap<string, u64>('b:');
const approves = new PersistentMap<string, u64>('a:');

const INIT_KEY = 'init';
const VERSION_KEY = 'version';
const METADATA_KEY = 'contract_metadata';

const CURRENT_VERSION = 'v3';
const TOTAL_SUPPLY: u64 = 1000000;

export function init(initialOwner: string): void {
    assert(storage.get<string>(VERSION_KEY) !== CURRENT_VERSION, `Token version ${CURRENT_VERSION} already initialized`);
    if (!storage.contains(INIT_KEY)) {
        balances.set(initialOwner, TOTAL_SUPPLY);
        storage.set(INIT_KEY, 'done');
    }
    storage.set(METADATA_KEY, new FTContractMetadata());
    storage.set(VERSION_KEY, CURRENT_VERSION);
}

export function totalSupply(): string {
    return TOTAL_SUPPLY.toString();
}

export function balanceOf(tokenOwner: string): u64 {
    if (!balances.contains(tokenOwner)) {
        return 0;
    }
    return balances.getSome(tokenOwner);
}

export function allowance(tokenOwner: string, spender: string): u64 {
    const key = `${tokenOwner}:${spender}`;
    if (!approves.contains(key)) {
        return 0;
    }
    return approves.getSome(key);
}

export function transfer(to: string, tokens: u64): boolean {
    const fromBalance = getBalance(context.sender);
    assert(fromBalance >= tokens, 'not enough tokens on account');
    balances.set(context.sender, fromBalance - tokens);
    balances.set(to, getBalance(to) + tokens);
    return true;
}

export function approve(spender: string, tokens: u64): boolean {
    approves.set(`${context.sender}:${spender}`, tokens);
    return true;
}

export function transferFrom(from: string, to: string, tokens: u64): boolean {
    const fromBalance = getBalance(from);
    assert(fromBalance >= tokens, 'not enough tokens on account');
    const approvedAmount = allowance(from, to);
    assert(approvedAmount >= tokens, 'not enough tokens approved to transfer');
    balances.set(from, fromBalance - tokens);
    balances.set(to, getBalance(to) + tokens);
    return true;
}

export function ft_metadata(): FTContractMetadata {
    return storage.getSome<FTContractMetadata>(METADATA_KEY)
}

function getBalance(owner: string): u64 {
    return balances.contains(owner) ? balances.getSome(owner) : 0;
}