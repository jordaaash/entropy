import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { Entropy } from '../target/types/entropy';

describe('entropy', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Entropy as Program<Entropy>;

  it('prime', async () => {
    // Add your test here.
    const tx = await program.rpc.prime({});
    console.log("Your transaction signature", tx);
  });
});
