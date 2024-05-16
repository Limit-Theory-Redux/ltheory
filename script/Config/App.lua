Config.org                       = 'LimitTheoryRedux'
Config.app                       = 'LimitTheoryRedux'

Config.orgInfo                   = {
    repository = "https://github.com/Limit-Theory-Redux/ltheory",
    discord = "https://discord.gg/MrfRR5ytJF",
    wiki = "https://wiki.ltredux.org",
    blog = "https://blog.ltredux.org",
    reddit = "https://www.reddit.com/r/LimitTheoryRedux/"
}

Config.gameTitle                 = "Limit Theory Redux"

Config.userInitFilename          = "user"
Config.userInitFiletype          = ".ini"

Config.timeToResetToSplashscreen = 60

Config.paths                     = {
    files         = Directory.GetPrefPath(Config.org, Config.app), -- base directory using environment-agnostic path
    soundAmbiance = "./res/sound/system/audio/music/",
    soundEffects  = "./res/sound/system/audio/fx/",
    enums         = "./script/Enums/",
    types         = "./script/Types/"
}

-- Initialize SubConfigTables
Config.audio                     = {}
Config.debug                     = {}
Config.econ                      = {}
Config.gen                       = {}
Config.game                      = {}
Config.render                    = {}
Config.ui                        = {}
Config.weapons                   = {}

function Config.getCurrentTimestamp()
    return os.time(os.date("!*t"))
end
