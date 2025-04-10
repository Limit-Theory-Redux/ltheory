-- All component types.
return {
    EntityHierarchy = require('Components.Core.EntityHierarchy'),
    EntityName = require('Components.Core.EntityName'),
    ExpiryComponent = require('Components.Economy.ExpiryComponent'),
    InventoryComponent = require('Components.Economy.InventoryComponent'),
    MarketplaceComponent = require('Components.Economy.MarketplaceComponent'),
    OrderItemTypeComponent = require('Components.Economy.OrderItemTypeComponent'),
    OrderStatusComponent = require('Components.Economy.OrderStatusComponent'),
    OwnershipComponent = require('Components.Economy.OwnershipComponent'),
    PlayerBankAccountComponent = require('Components.Economy.PlayerBankAccountComponent'),
    PlayerListComponent = require('Components.Economy.PlayerListComponent'),
    PriceComponent = require('Components.Economy.PriceComponent'),
    QuantityComponent = require('Components.Economy.QuantityComponent'),
    SeedComponent = require('Components.Generation.SeedComponent'),
    MassComponent = require('Components.Physics.MassComponent'),
    RigidBodyComponent = require('Components.Physics.RigidBodyComponent'),
    TransformComponent = require('Components.Physics.TransformComponent'),
    CameraData = require('Components.Rendering.CameraData'),
    Effect = require('Components.Rendering.Effect'),
    RenderComponent = require('Components.Rendering.RenderComponent'),
    ShapeComponent = require('Components.Spatial.ShapeComponent')
}