/// <reference types="Cypress" />

describe("The app", () => {
    it("should work", () => {
        cy.visit("/");

        // just some assertions
        cy.contains("Swap");
        cy.contains("Create wallet");

        // create wallet procedure
        cy.get("[data-cy=wallet-create]").click();
        cy.get("[data-cy=wallet-password]").type("foo");
        cy.get("[data-cy=create-wallet-form]").submit();

        // open wallet to get address
        cy.get("[data-cy=wallet-info]").click();
        cy.get("[data-cy=wallet-address]").invoke("text").then(async (address) => {
            // should be regtest format: TODO maybe no assert needed
            expect(address).to.match(/el*/);

            cy.request("POST", `/api/faucet/${address}`);
        });
        cy.get("[data-cy=wallet-address]").type("{esc}");

        // swap
        cy.get("[data-cy=Alpha-amount]").find("input").clear();
        cy.get("[data-cy=Alpha-amount]").find("input").type("0.42");
        // cy.get("[data-cy=swap-button]").click();

        // Sign with wallet
        // TODO verify all numbers
        cy.get("[data-cy=swap-button]", { timeout: 20 * 1000 }).should("be.enabled");
        cy.get("[data-cy=swap-button]").click();

        cy.get("[data-cy=sign-and-send]").click();

        cy.url().should("include", "/swapped/");
    });
});
