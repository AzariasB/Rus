module Session exposing (Data, default, removeFlash, setFlash)

import Browser.Navigation as Nav
import Redirection exposing (Redirection)
import Url


type alias Data =
    { url : Url.Url
    , nav : Nav.Key
    , redirections : Maybe (List Redirection)
    , flash : Maybe String
    }


default : Url.Url -> Nav.Key -> Data
default url nav =
    Data url nav Nothing Nothing


setFlash : Data -> String -> Data
setFlash data flash =
    { data | flash = Just flash }


removeFlash : Data -> Data
removeFlash data =
    { data | flash = Nothing }
