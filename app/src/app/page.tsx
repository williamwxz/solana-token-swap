// app/src/app/page.tsx
import React, { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { getProgram } from '../utils/anchorClient';
import idl from '../idl.json';
import { PublicKey } from '@solana/web3.js';

const programId = new PublicKey('F7T2naGX3R3izoV96t84788Wru2foc8csf7AvcTwuCH4');

const HomePage = () => {
  const { publicKey, signTransaction } = useWallet();
  const [message, setMessage] = useState('');

  const initializePool = async () => {
    if (!publicKey) {
      setMessage('Please connect your wallet!');
      return;
    }

    try {
      const program = await getProgram({ publicKey, signTransaction }, programId, idl);
      // Call your initializePool method here
      // Example: await program.methods.initializePool(...).accounts({...}).rpc();
      setMessage('Pool initialized successfully!');
    } catch (error) {
      console.error(error);
      setMessage('Failed to initialize pool.');
    }
  };

  return (
    <div>
      <h1>Solana Token Swap</h1>
      <button onClick={initializePool}>Initialize Pool</button>
      <p>{message}</p>
    </div>
  );
};

export default HomePage;