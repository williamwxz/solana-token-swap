import * as anchor from '@coral-xyz/anchor';
import { connection } from './solanaConnection';

export const getProvider = (wallet: any) => {
  const provider = new anchor.AnchorProvider(connection, wallet, {
    preflightCommitment: 'processed',
  });
  return provider;
};

export const getProgram = async (wallet: any, programId: any, idl: any) => {
  const provider = getProvider(wallet);
  const program = new anchor.Program(idl, programId, provider);
  return program;
};