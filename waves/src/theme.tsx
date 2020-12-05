import { Button, extendTheme } from "@chakra-ui/react";
import React from "react";

const customIcons = {
    bitcoin: {
        path: (
            <g transform="translate(.0063088 -.0030198)">
                <path
                    d="m63.033 39.744c-4.274 17.143-21.637 27.576-38.782 23.301-17.138-4.274-27.571-21.638-23.295-38.78 4.272-17.145 21.635-27.579 38.775-23.305 17.144 4.274 27.576 21.64 23.302 38.784z"
                    fill="#f7931a"
                />
                <path
                    d="m46.103 27.444c0.637-4.258-2.605-6.547-7.038-8.074l1.438-5.768-3.511-0.875-1.4 5.616c-0.923-0.23-1.871-0.447-2.813-0.662l1.41-5.653-3.509-0.875-1.439 5.766c-0.764-0.174-1.514-0.346-2.242-0.527l4e-3 -0.018-4.842-1.209-0.934 3.75s2.605 0.597 2.55 0.634c1.422 0.355 1.679 1.296 1.636 2.042l-1.638 6.571c0.098 0.025 0.225 0.061 0.365 0.117-0.117-0.029-0.242-0.061-0.371-0.092l-2.296 9.205c-0.174 0.432-0.615 1.08-1.609 0.834 0.035 0.051-2.552-0.637-2.552-0.637l-1.743 4.019 4.569 1.139c0.85 0.213 1.683 0.436 2.503 0.646l-1.453 5.834 3.507 0.875 1.439-5.772c0.958 0.26 1.888 0.5 2.798 0.726l-1.434 5.745 3.511 0.875 1.453-5.823c5.987 1.133 10.489 0.676 12.384-4.739 1.527-4.36-0.076-6.875-3.226-8.515 2.294-0.529 4.022-2.038 4.483-5.155zm-8.022 11.249c-1.085 4.36-8.426 2.003-10.806 1.412l1.928-7.729c2.38 0.594 10.012 1.77 8.878 6.317zm1.086-11.312c-0.99 3.966-7.1 1.951-9.082 1.457l1.748-7.01c1.982 0.494 8.365 1.416 7.334 5.553z"
                    fill="#ffffff"
                />
            </g>
        ),
        viewBox: "0 0 64 64",
    },
    usdt: {
        path: (
            <g>
                <polygon
                    fill="#343434"
                    points="127.9611 0 125.1661 9.5 125.1661 285.168 127.9611 287.958 255.9231 212.32"
                />
                <polygon
                    fill="#8C8C8C"
                    points="127.962 0 0 212.32 127.962 287.959 127.962 154.158"
                />
                <polygon
                    fill="#3C3C3B"
                    points="127.9611 312.1866 126.3861 314.1066 126.3861 412.3056 127.9611 416.9066 255.9991 236.5866"
                />
                <polygon
                    fill="#8C8C8C"
                    points="127.962 416.9052 127.962 312.1852 0 236.5852"
                />
                <polygon
                    fill="#141414"
                    points="127.9611 287.9577 255.9211 212.3207 127.9611 154.1587"
                />
                <polygon
                    fill="#393939"
                    points="0.0009 212.3208 127.9609 287.9578 127.9609 154.1588"
                />
            </g>
        ),
        viewBox: "0 0 256 417",
    },
};

const theme = extendTheme({
    icons: {
        ...customIcons,
    },
    textStyles: {
        actionable: {
            fontSize: "lg",
            color: "gray.500",
        },
        info: {
            fontSize: "sm",
            color: "gray.500",
        },
    },
    swapButton: {
        baseStyle: {
            colorScheme: "teal",
            size: "lg",
            bg: "#304FFE",
            rounded: "md",
            width: "200px",
            // _hover: {{ bg: "blue.300" }},
        },
    },
    components: {
        Button: {
            baseStyle: {
                bg: "#304FFE",
                fontWeight: "bold",
                fontColor: "green",
                color: "green",
                _hover: {
                    bg: "blue.300",
                },
            },

            sizes: {
                lg: {
                    h: "56px",
                    fontSize: "lg",
                    px: "32px",
                },
            },
            // Custom variant
            variants: {
                "main_button": {
                    h: "50px",
                    w: "300px",
                    color: "white",
                },
                "wallet_button": {
                    color: "white",
                },
            },
        },
    },
});

export default theme;
