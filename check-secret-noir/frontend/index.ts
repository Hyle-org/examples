import { NodeApiHttpClient } from "hyle";
import { build_blob_transaction, build_proof_transaction } from "./lib";

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
      ".check_secret";
    const password = (document.getElementById("password") as HTMLInputElement)
      ?.value;

    if (!identity || !password) {
      show("logs", "Identity and password are required.");
      return;
    }

    show("logs", "Building blob transaction... ⏳");
    const blobTx = await build_blob_transaction(identity, password);
    const tx_hash = await node.sendBlobTx(blobTx);
    show("logs", "Register transaction sent... ✅");

    show("logs", "Building proof transaction... ⏳");
    const before = Date.now();
    const proofTx = await build_proof_transaction(identity, password, tx_hash);
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
