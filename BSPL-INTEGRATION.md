
## Annotated BSPL

### Notes
* BSPL does not define a stepwise scenarios, any functions can be called in any order by the respective role

### Example
a vanilla BSPL payload:
```
Purchase {
 roles B, S, Shipper
 parameters out ID key, out item, out price, out outcome
 private address, shipped, accept, reject, resp

 B -> S: rfq[out ID, out item]
 S -> B: quote[in ID, in item, out price]
 B -> S: accept[in ID, in item, in price, out address, out resp, out accept]
 B -> S: reject[in ID, in item, in price, out outcome, out resp, out reject]

 S -> Shipper: ship[in ID, in item, in address, out shipped]
 Shipper -> B: deliver[in ID, in item, in address, out outcome]
}
```

the same BSPL payload with semantic annotations (comments are there just for convenience, they are not necessary for the protocol to be parsed correctly):

```
// The protocol semantically annotated
Purchase <Protocol>("The process of a buyer acquiring an item from a seller in exchange for payment, with optional third-party shipping.") {

    // --- Role Definitions ---
    // Each role is defined with its type and a clear description of its part in the protocol.
    roles
        B <Agent>("the party wanting to buy an item"),
        S <Agent>("the party selling the item"),
        Shipper <Agent>("the third-party entity responsible for logistics and delivery")

    // --- Parameter Definitions ---
    // All data fields exchanged in the protocol are defined with their type and semantic meaning.
    // The BSPL 'key' attribute is preserved.
    parameters
        ID key <String>("a unique identifier for the transaction"),
        item <String>("the name or description of the product being purchased"),
        price <Float>("the cost of the item quoted by the seller"),
        address <String>("the physical destination for shipping the item"),
        shipped <Bool>("a confirmation status indicating the item has been dispatched by the seller"),
        accept <Bool>("a boolean flag indicating the buyer agrees to the quote"),
        reject <Bool>("a boolean flag indicating the buyer declines the quote"),
        resp <String>("a textual response or reason accompanying an accept/reject decision"),
        outcome <String>("a final status message describing the result of the protocol")

    // --- Interaction Protocol ---
    // Each interaction is now a semantically annotated message, defining its specific intent.

    // Buyer requests a quote from the Seller.
    B -> S: rfq <Action>("request for a price quote")[out ID, out item]

    // Seller responds with a price.
    S -> B: quote <Action>("provide a price quote for a requested item")[in ID, in item, out price]
    
    // Buyer accepts the quote.
    B -> S: accept <Action>("accept the seller's price quote and provide shipping details")[in ID, in item, in price, out address, out resp, out accept]

    // Buyer rejects the quote.
    B -> S: reject <Action>("reject the seller's price quote with a reason")[in ID, in item, in price, out outcome, out resp, out reject]

    // If accepted, Seller instructs the Shipper.
    S -> Shipper: ship <Action>("request shipment of the purchased item to an address")[in ID, in item, in address, out shipped]
    
    // Shipper completes the delivery to the Buyer.
    Shipper -> B: deliver <Action>("confirm delivery of the item to the buyer and finalize the outcome")[in ID, in item, in address, out outcome]
}
```

### Example: Nested BSPL

Vanilla BSPL:
```
Logistics  {
 roles M, W, P, L, S, C // Merchant, Warehouse, Packer, Loader, Scanner, Courier
 parameters out ID key, out order, out delivery
 private tag, package, route

 M -> W: NotifyOrder[out ID key, out order]
 Pack(W, P, S, in ID key, in order, out tag, out package)
 Load(W, L, S, C, in ID key, in order, in tag, out route)
 W -> M: Deliver[in ID key, in route, out delivery]
}

Pack {
  roles W, P, S // Warehouse, Packer, Scanner
  parameters in ID key, in order, out tag, out package
  private data, written

  W -> P: Pack[in ID key, in order]
  P -> W: Tag[in ID key, in order, out tag]
  P -> S: WriteTag[in ID key, in order, in tag, out data]
  S -> P: TagWritten[in ID key, in tag, in data, out written]
  P -> W: Packed[in ID key, in written, out package]
}

Load {
  roles W, L, S, C // Warehouse, Loader, Scanner, Courier
  parameters in ID key, in order, in tag, out route
  private type, vehicle, tag, find, found, package, loaded

  W -> L: Load[in ID key, in order, in tag]

  L -> C: GetVehicle[in ID key, in order, out type]
  C -> L: SendVehicle[in ID key, in type, out vehicle]

  L -> S: FindPackage[in ID key, in tag, out find]
  S -> L: FoundPackage[in ID key, in tag, out package]
  L -> C: LoadVehicle[in ID key, in package, in vehicle, out loaded]

  C -> W: Enroute[in ID key, in vehicle, in package, out route]
}
```

The same BSPL payload with semantic annotations
```
// --- Top-Level Protocol: Logistics ---
// This protocol describes the entire order fulfillment process, orchestrating two sub-protocols.
Logistics <Protocol>("The end-to-end process of fulfilling a customer order, from initial notification to final delivery confirmation.") {
    roles
        M <Agent>("a merchant who owns the goods"),
        W <Agent>("a central storage and fulfillment facility"),
        P <Agent>("an employee or robot that packages items"),
        L <Agent>("an employee or robot that loads packages onto vehicles"),
        S <Agent>("an automated scanning system for tracking items"),
        C <Agent>("a courier service responsible for final transport")

    parameters
        ID key <String>("the unique identifier for the customer order"),
        order <String>("a description of the items included in the order"),
        tag <String>("a unique scannable tag generated for the physical package"),
        package <String>("a reference to the final, physically packed box or container"),
        route <String>("the planned delivery route for the courier's vehicle"),
        delivery <String>("a final confirmation or tracking number for the completed delivery")

    // --- Protocol Flow ---
    // A simple message interaction to kick off the process.
    M -> W: NotifyOrder <Action>("inform the warehouse of a new customer order to be fulfilled")[out ID, out order]

    // A nested protocol call to the 'Pack' sub-protocol.
    // It maps roles from Logistics (W, P, S) and the necessary data in and out.
    Pack(W, P, S)[in ID, in order, out tag, out package]

    // A second nested protocol call to the 'Load' sub-protocol.
    Load(W, L, S, C)[in ID, in order, in tag, out route]

    // A final message to confirm the process has completed.
    W -> M: Deliver <Action>("confirm that the order is out for delivery with the courier")[in ID, in route, out delivery]
}


// --- Sub-Protocol 1: Pack ---
// This protocol details the specific steps involved in packaging an order.
Pack <SubProtocol>("The process of picking items for an order and packaging them into a tagged container.") {
    roles
        W <Agent>("the warehouse coordinator overseeing the packing process"),
        P <Agent>("the packing agent responsible for physically packing items"),
        S <Agent>("the tag scanning system used to verify package contents and data")

    parameters
        ID key <String>("the order identifier being processed"),
        order <String>("the list of items to be packed"),
        tag <String>("the generated scannable tag for the package"),
        package <String>("a reference to the final, sealed package"),
        data <String>("the raw order data to be encoded into the physical tag"),
        written <Bool>("a confirmation status that the tag data was written successfully")

    // --- Interaction Flow within Pack ---
    W -> P: Pack <Action>("instruct the packer to begin packing the items for an order")[in ID, in order]
    P -> W: Tag <Action>("request a unique, scannable tag for the newly created package")[in ID, in order, out tag]
    P -> S: WriteTag <Action>("command the scanner system to write order data to the physical tag")[in ID, in order, in tag, out data]
    S -> P: TagWritten <Action>("confirm from the scanner that the tag data has been successfully written")[in ID, in tag, in data, out written]
    P -> W: Packed <Action>("confirm to the warehouse that the order is fully packed and correctly tagged")[in ID, in written, out package]
}


// --- Sub-Protocol 2: Load ---
// This protocol details the steps for getting a packed item onto a courier's vehicle.
Load <SubProtocol>("The process of retrieving a packed item, assigning a vehicle, and loading it for delivery.") {
    roles
        W <Agent>("the warehouse coordinator for loading and dispatch"),
        L <Agent>("the loading agent who physically moves packages"),
        S <Agent>("the scanning system used to locate packages"),
        C <Agent>("the courier who provides and operates the delivery vehicle")

    parameters
        ID key <String>("the order identifier being loaded"),
        order <String>("the details of the order for reference"),
        tag <String>("the scannable tag used to locate the package"),
        package <String>("the reference to the physical package being loaded"),
        route <String>("the final delivery route assigned to the vehicle"),
        type <String>("the type of vehicle required for the delivery, e.g., 'refrigerated truck'"),
        vehicle <String>("a specific identifier for the assigned vehicle"),
        find <Bool>("a status flag indicating a request to find a package"),
        found <Bool>("a status flag indicating a package has been located"),
        loaded <Bool>("a confirmation status that the package is securely on the vehicle")
        
    // --- Interaction Flow within Load ---
    W -> L: Load <Action>("instruct the loader to begin the loading process for a packed order")[in ID, in order, in tag]
    L -> C: GetVehicle <Action>("request a suitable vehicle from the courier for the order")[in ID, in order, out type]
    C -> L: SendVehicle <Action>("assign and send a specific vehicle to the loading bay")[in ID, in type, out vehicle]
    L -> S: FindPackage <Action>("use the scanner to locate the physical package in the warehouse")[in ID, in tag, out find]
    S -> L: FoundPackage <Action>("confirm the package has been located")[in ID, in tag, in find, out package]
    L -> C: LoadVehicle <Action>("command the courier to load the found package onto the assigned vehicle")[in ID, in package, in vehicle, out loaded]
    C -> W: Enroute <Action>("confirm the vehicle is loaded and has begun its delivery route")[in ID, in vehicle, in package, out route]
}
```