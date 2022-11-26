module Main exposing (Model, init, main, update, view)

import Browser exposing (Document)
import Browser.Navigation as Nav
import Page
import Url exposing (Url)



-- MAIN


main =
    Browser.application
        { init = init
        , update = update
        , subscriptions = \_ -> Sub.none
        , view = view
        , onUrlRequest = Page.ClickedLink
        , onUrlChange = Page.ChangedUrl
        }



--Browser.Navigation.load href
-- MODEL


type alias Model =
    { page : Page.Model }


init : () -> Url -> Nav.Key -> ( Model, Cmd Page.Msg )
init _ url key =
    let
        ( page, cmd ) =
            Page.init url key
    in
    ( { page = page }, cmd )



-- UPDATE


update : Page.Msg -> Model -> ( Model, Cmd Page.Msg )
update msg { page } =
    let
        ( res, cmd ) =
            Page.update msg page
    in
    ( { page = res }, cmd )



-- VIEW


view : Model -> Document Page.Msg
view { page } =
    Page.view page
