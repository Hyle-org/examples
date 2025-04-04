import { borshSerialize, BorshSchema, borshDeserialize } from "borsher";
import { Blob } from "hyle";

export const sidContractName = "sid";

//
// Types
//

export type IdentityAction =
  | {
      RegisterIdentity: {
        account: string;
      };
    }
  | {
      VerifyIdentity: {
        nonce: number;
        account: string;
      };
    };

//
// Builders
//

export const register = (account: string): Blob => {
  const action: IdentityAction = {
    RegisterIdentity: { account },
  };
  const blob: Blob = {
    contract_name: sidContractName,
    data: serializeIdentityAction(action),
  };
  return blob;
};

export const verifyIdentity = (nonce: number, account: string): Blob => {
  const action: IdentityAction = {
    VerifyIdentity: { nonce, account },
  };

  const blob: Blob = {
    contract_name: sidContractName,
    data: serializeIdentityAction(action),
  };
  return blob;
};

//
// Serialisation
//

const serializeIdentityAction = (action: IdentityAction): number[] => {
  return Array.from(borshSerialize(schema, action));
};
export const deserializeIdentityAction = (data: number[]): IdentityAction => {
  return borshDeserialize(schema, Buffer.from(data));
};

const schema = BorshSchema.Enum({
  RegisterIdentity: BorshSchema.Struct({
    account: BorshSchema.String,
  }),
  VerifyIdentity: BorshSchema.Struct({
    account: BorshSchema.String,
    nonce: BorshSchema.u128,
  }),
});
