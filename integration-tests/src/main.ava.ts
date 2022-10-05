import { Worker, NearAccount, NEAR } from "near-workspaces";
import anyTest, { TestFn } from "ava";

const test = anyTest as TestFn<{
  worker: Worker;
  accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async (t) => {
  // Init the worker and start a Sandbox server
  const worker = await Worker.init();

  // Get root account
  const root = worker.rootAccount;

  // Create test accounts
  const alice = await root.createSubAccount("alice");
  const beneficiary = await root.createSubAccount("beneficiary");
  const factory = await root.createSubAccount("factory");

  // Deploy factory contract
  await factory.deploy(process.argv[2]);

  // Save state for test runs, it is unique for each test
  t.context.worker = worker;
  t.context.accounts = {
    factory,
    alice,
    beneficiary,
  };
});

test.afterEach(async (t) => {
  await t.context.worker.tearDown().catch((error) => {
    console.log("Failed tear down the worker:", error);
  });
});

test("create_factory_subaccount_and_deploy tests", async (t) => {
  const { factory, alice, beneficiary } = t.context.accounts;

  let create = await alice.call(
    factory,
    "create_factory_subaccount_and_deploy",
    { name: `sub`, beneficiary: beneficiary },
    { gas: "80000000000000", attachedDeposit: NEAR.parse("1.24 N").toString() }
  );

  t.is(create, true);
});
