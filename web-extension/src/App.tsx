import { Box, ChakraProvider, Heading } from "@chakra-ui/react";
import * as React from "react";
import { useAsync } from "react-async";
import {
    cancelLoan,
    cancelSwap,
    getLoanToSign,
    getSwapToSign,
    getWalletBalance,
    getWalletStatus
} from "./background-proxy";
import AddressQr from "./components/AddressQr";
import WalletBalances from "./components/Balances";
import ConfirmLoan from "./components/ConfirmLoan";
import CreateOrUnlockWallet from "./components/CreateOrUnlockWallet";
import { Status } from "./models";
import theme from "./theme";
import ConfirmSwap from "./components/ConfirmSwap";

const App = () => {
    const walletStatusHook = useAsync({ promiseFn: getWalletStatus });
    const walletBalanceHook = useAsync({ promiseFn: getWalletBalance });
    const swapToSignHook = useAsync({ promiseFn: getSwapToSign });
    const loanToSignHook = useAsync({ promiseFn: getLoanToSign });

    let { data: walletStatus, reload: reloadWalletStatus } = walletStatusHook;
    let { data: balanceUpdates, reload: reloadWalletBalances } = walletBalanceHook;
    let { data: swapToSign, reload: reloadSwapToSing } = swapToSignHook;
    let { data: loanToSign, reload: reloadLoanToSing } = loanToSignHook;

    const refreshAll = async () => {
        await reloadWalletBalances();
        await reloadWalletStatus();
        await reloadSwapToSing();
        await reloadLoanToSing();
    };

    // we want to either sign a swap or the loan but not both:
    let signLoan = false;
    if (!swapToSign && loanToSign) {
        signLoan = true;
    }

    return (
        <ChakraProvider theme={theme}>
            <Box h={600} w={400}>
                {walletStatus?.status === Status.Loaded
                    && <>
                        {balanceUpdates && <WalletBalances balanceUpdates={balanceUpdates} />}
                        <AddressQr />
                        {swapToSign && <ConfirmSwap
                            onCancel={async () => {
                                await cancelSwap(swapToSign!);
                                await refreshAll();
                            }}
                            onSuccess={refreshAll}
                            swapToSign={swapToSign!}
                        />}
                        {signLoan
                            && <ConfirmLoan
                                onCancel={async () => {
                                    await cancelLoan(loanToSign!);
                                    await refreshAll();
                                }}
                                onSuccess={refreshAll}
                                loanToSign={loanToSign!}
                            />}
                    </>}
                {walletStatus?.status === Status.NotLoaded
                    && <>
                        <Heading>Unlock Wallet</Heading>
                        <CreateOrUnlockWallet
                            onUnlock={refreshAll}
                            status={Status.NotLoaded}
                        />
                    </>}

                {walletStatus?.status === Status.None
                    && <>
                        <Heading>Create Wallet</Heading>
                        <CreateOrUnlockWallet
                            onUnlock={refreshAll}
                            status={Status.None}
                        />
                    </>}
            </Box>
        </ChakraProvider>
    );
};

export default App;
