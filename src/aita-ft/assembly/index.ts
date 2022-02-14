import { context, storage, logging, PersistentMap } from 'near-sdk-as';

const balances = new PersistentMap<string, u64>('b:');
const approves = new PersistentMap<string, u64>('a:');

const TOTAL_SUPPLY: u64 = 1000000;

export function init(initialOwner: string): void {
    logging.log(`initialOwner: ${initialOwner}`);
    assert(storage.get<string>('init') === null, 'Token totalSupply already initialized');
    balances.set(initialOwner, TOTAL_SUPPLY);
    storage.set('init', 'done');
}

export function totalSupply() {
    return TOTAL_SUPPLY.toString();
}

export function balanceOf(tokenOwner: string): u64 {
    logging.log(`balanceOf: ${tokenOwner}`);
    if (!balances.contains(tokenOwner)) {
        return 0;
    }
    return balances.getSome(tokenOwner);
}

export function allowance(tokenOwner: string, spender: string): u64 {
    const key = `${tokenOwner}${spender}`;
    if (!approves.contains(key)) {
        return 0;
    }
    return approves.getSome(key);
}

export function transfer(to: string, tokens: u64): boolean {
    logging.log(`transfer from ${context.sender} to ${to} tokens ${tokens}`);
    const fromBalance = getBalance(context.sender);
    assert(fromBalance >= tokens, 'not enough tokens on account');
    balances.set(context.sender, fromBalance - tokens);
    balances.set(to, getBalance(to) + tokens);
    return true;
}

export function approve(spender: string, tokens: u64): boolean {
    logging.log(`approve spender: ${spender} tokens ${tokens.toString()}`);
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

function getBalance(owner: string): u64 {
    return balances.contains(owner) ? balances.getSome(owner) : 0;
}