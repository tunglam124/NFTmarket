# 🎨 Soroban NFT Marketplace

## 📖 Description
This project is a decentralized Smart Contract built on the **Stellar Soroban** network. 
The main objective of this project is to build a comprehensive NFT Marketplace ecosystem where users can easily mint, trade, and auction digital assets transparently and securely. The contract features an automated revenue-sharing mechanism (distributing platform fees and creator royalties) and integrates an internal Escrow system to ensure trust and safety for both buyers and sellers.

## ✨ Features
Below are the core functionalities integrated into the Smart Contract:

* **🖌️ Mint NFT:** Allows users (Creators) to mint new NFTs by providing metadata (URI, Title), setting an initial price, and defining a royalty percentage (Royalty BPS).
* **🏷️ List / Delist:** NFT owners have the right to list their NFTs for public sale or delist them from the marketplace at any time.
* **🛒 Direct Buy:** Users can instantly purchase listed NFTs. The contract automatically processes the payment and distributes the funds: paying the seller, sending royalties to the creator, and transferring the trading fee to the platform admin.
* **⚖️ Auction System:**
    * Sellers can start an auction by setting a minimum bid and a specific end time.
    * The system automatically refunds the previous highest bidder as soon as a new, higher bid is placed.
    * Secure Settlement: Once the auction ends, the contract transfers NFT ownership to the winner and allocates the funds accordingly.
* **💰 Internal Escrow:** A built-in wallet system that allows users to securely deposit and withdraw funds (in stroops) before participating in trades or auctions.
* **🎁 Transfer:** Enables owners to directly transfer or gift their NFTs to another wallet address without requiring a sale transaction.
* **⚙️ Admin & Governance:** Role-based access control that allows the Admin to initialize the marketplace and configure the platform fee rate (Fee BPS).

# Contract
Contract link : https://stellar.expert/explorer/testnet/tx/595fd7c84c83c05dae14b131d4efeffde067fb570ebd97d507611ded379ee141
<img width="1918" height="1078" alt="image" src="https://github.com/user-attachments/assets/52548469-7b96-47b6-b551-7e41f8ee281d" />

## 🚀 Future Scopes
While the current smart contract provides a solid foundation for an NFT Marketplace, there are several exciting features and optimizations planned for future updates:

* **Stellar Asset Contract (SAC) Integration:** Transitioning from the internal Escrow system to direct payments using native Stellar tokens (such as real XLM or USDC) via Soroban Token Interface.
* **Batch Operations:** Adding support for batch minting and batch listing to optimize compute/gas fees and improve user experience for large creators.
* **Frontend dApp Integration:** Developing a user-friendly web interface (using React.js or Next.js) connected to the Freighter wallet for seamless on-chain interactions.
* **Multiple NFT Collections:** Extending the smart contract to support a factory pattern, allowing users to deploy and manage their own independent NFT collections.
* **Auction Enhancements:** Adding a `cancel_auction` function to allow sellers to safely close auctions if no bids have been placed.

## 👨‍💻 Profile
**[Vũ Lâm]**  

* **Core Skills:** Rust, Stellar Soroban, Web3 Development, Smart Contract Auditing.
* **GitHub:** https://github.com/tunglam124
* **LinkedIn:** https://www.linkedin.com/in/v%C5%A9-t%C3%B9ng-l%C3%A2m-undefined-b67175311/
* **Email:** vulam.061204@gmail.com

---

