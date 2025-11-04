return {
    Age = require("Modules.CelestialObjects.Components.AgeComponent"),
    Atmosphere = require("Modules.CelestialObjects.Components.AtmosphereComponent"),
    Composition = require("Modules.CelestialObjects.Components.CompositionComponent"),
    Density = require("Modules.CelestialObjects.Components.DensityComponent"),
    Eccentricity = require("Modules.CelestialObjects.Components.EccentricityComponent"),
    Gravity = require("Modules.CelestialObjects.Components.GravityComponent"),
    Luminosity = require("Modules.CelestialObjects.Components.LuminosityComponent"),
    MagneticField = require("Modules.CelestialObjects.Components.MagneticFieldComponent"),
    Metallicity = require("Modules.CelestialObjects.Components.MetallicityComponent"),
    RotationPeriod = require("Modules.CelestialObjects.Components.RotationPeriodComponent"),
    Stability = require("Modules.CelestialObjects.Components.StabilityComponent"),
    Temperature = require("Modules.CelestialObjects.Components.TemperatureComponent"),
    Thickness = require("Modules.CelestialObjects.Components.ThicknessComponent"),

    Gen = {
        Planet = require("Modules.CelestialObjects.Components.Gen.PlanetGenComponent"),
        PlanetRing = require("Modules.CelestialObjects.Components.Gen.PlanetRingGenComponent"),

    },

    Simulation = {
        CloudMotion = require("Modules.CelestialObjects.Components.Simulation.CloudMotionComponent"),
        PlanetaryRingMotion = require("Modules.CelestialObjects.Components.Simulation.PlanetaryRingMotionComponent")
    }
}
