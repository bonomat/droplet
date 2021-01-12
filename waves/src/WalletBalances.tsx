import { Box, Button, HStack, Image, Text } from "@chakra-ui/react";
import React from "react";
import { Balances } from "./App";
import Btc from "./components/bitcoin.svg";
import Usdt from "./components/tether.svg";

interface WalletBalancesProps {
    balances: Balances;
    onClick: () => void;
}

export default function WalletBalances({ balances, onClick }: WalletBalancesProps) {
    return <Box as={Button} onClick={onClick} bg="#FFFFFF" data-cy="wallet-info">
        <HStack align="left">
            <Box>
                <HStack>
                    <Box>
                        <Image src={Usdt} h="20px" />
                    </Box>
                    <Box>
                        <Text textStyle="info" data-cy="usdt-balance">L-USDT: {balances.usdt}</Text>
                    </Box>
                </HStack>
            </Box>
            <Box>
                <HStack>
                    <Box>
                        <Image src={Btc} h="20px" />
                    </Box>
                    <Box>
                        <Text textStyle="info" data-cy="btc-balance">L-BTC: {balances.btc}</Text>
                    </Box>
                </HStack>
            </Box>
        </HStack>
    </Box>;
}
