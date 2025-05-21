import { BlobTransaction, NodeApiHttpClient } from "hyli";
import { build_blob, build_proof_transaction, register_contract } from "./lib";

const node = new NodeApiHttpClient("http://127.0.0.1:4321");

const show = (id: string, content: string): void => {
  const container = document.getElementById(id);
  if (container) {
    container.appendChild(document.createTextNode(content));
    container.appendChild(document.createElement("br"));
  }
};

document.getElementById("submit")?.addEventListener("click", async () => {
  try {
    // Prepare inputs
    const identity =
      (document.getElementById("identity") as HTMLInputElement)?.value +
      "@check_secret";
    const password = (document.getElementById("password") as HTMLInputElement)
      ?.value;

    if (!identity || !password) {
      show("logs", "Identity and password are required.");
      return;
    }

    await register_contract(node);

    show("logs", "Building blob transaction... ⏳");
    const blob = await build_blob(identity, password);
    const blobTx: BlobTransaction = {
      identity: identity,
      blobs: [blob],
    };
    const tx_hash = await node.sendBlobTx(blobTx);
    show("logs", "Register transaction sent... ✅");

    show("logs", "Building proof transaction... ⏳");
    const before = Date.now();
    const proofTx = await build_proof_transaction(
      identity,
      password,
      tx_hash,
      0,
      1,
    );
    const after = Date.now();
    const time = after - before;
    show(
      "logs",
      `Generated proof transaction in ${(time / 1000).toFixed(2)}sec... ✅`,
    );

    await node.sendProofTx(proofTx);
    show("logs", "Proof sent... ✅");
  } catch (err: unknown) {
    console.error(err);
    const errorMessage =
      err instanceof Error ? err.message : "Unknown error occurred";
    show("logs", "Error: " + errorMessage);
  }
});
