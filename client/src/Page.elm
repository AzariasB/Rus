module Page exposing (Model, Msg(..), init, update, view)

import Browser exposing (Document)
import Browser.Navigation as Nav
import Html exposing (Html, a, div, footer, h1, header, text)
import Html.Attributes exposing (class, href)
import Pages.Create as C
import Pages.Home as H
import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, oneOf, s, top)


type alias Model =
    { page : Page
    , nav : Nav.Key
    , url : Url
    }


type Page
    = Home H.Model
    | Create C.Model


type Msg
    = ChangedUrl Url
    | ClickedLink Browser.UrlRequest
    | GotHomeMsg H.Msg
    | GotCreateMsg C.Msg


init : Url -> Nav.Key -> ( Model, Cmd Msg )
init url key =
    let
        ( homeModel, _ ) =
            H.init
    in
    route url { page = Home homeModel, nav = key, url = url }



-- VIEW


view : Model -> Document Msg
view { page, nav } =
    case page of
        Home model ->
            { title = "Home"
            , body = viewHeader :: Html.map GotHomeMsg (H.view model) :: [ viewFooter ]
            }

        Create model ->
            { title = "Create"
            , body = viewHeader :: Html.map GotCreateMsg (C.view model) :: [ viewFooter ]
            }

        _ ->
            { title = ""
            , body = viewHeader :: [ viewFooter ]
            }


viewHeader : Html msg
viewHeader =
    header []
        [ div [ class "container" ]
            [ h1 [] [ text "Rus" ]
            ]
        ]


viewFooter : Html msg
viewFooter =
    footer []
        [ div [ class "container" ]
            [ a [ class "logo-font", href "/" ] [ text "Home" ] ]
        ]



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg ({ page, nav } as model) =
    case ( msg, page ) of
        ( GotHomeMsg homeMsg, Home h ) ->
            H.update homeMsg h |> updateWith model Home GotHomeMsg

        ( GotCreateMsg createMsg, Create c ) ->
            C.update createMsg c |> updateWith model Create GotCreateMsg

        ( ClickedLink req, _ ) ->
            case req of
                Browser.External href ->
                    ( model, Nav.load href )

                Browser.Internal url ->
                    ( model, Nav.pushUrl nav (Url.toString url) )

        ( ChangedUrl url, _ ) ->
            route url model

        ( _, _ ) ->
            ( model, Cmd.none )


updateWith : Model -> (subModel -> Page) -> (subMsg -> Msg) -> ( subModel, Cmd subMsg ) -> ( Model, Cmd Msg )
updateWith model toModel toMsg ( subModel, subCmd ) =
    ( { model | page = toModel subModel }, Cmd.map toMsg subCmd )


route : Url -> Model -> ( Model, Cmd Msg )
route url model =
    let
        parser =
            oneOf
                [ pageRoute top (homeRoute model)
                , pageRoute (s "create") (createRoute model)
                ]
    in
    case Parser.parse parser <| Maybe.withDefault url <| Url.fromString <| String.replace "#" "" <| Url.toString url of
        Just answer ->
            answer

        Nothing ->
            ( model, Nav.load (Url.toString url) )


pageRoute : Parser a b -> a -> Parser (b -> c) c
pageRoute parser handler =
    Parser.map handler parser


homeRoute : Model -> ( Model, Cmd Msg )
homeRoute model =
    let
        ( hModel, hMsg ) =
            H.init
    in
    ( { model | page = Home hModel }, Cmd.map GotHomeMsg hMsg )


createRoute : Model -> ( Model, Cmd Msg )
createRoute model =
    let
        ( cModel, cMsg ) =
            C.init
    in
    ( { model | page = Create cModel }, Cmd.map GotCreateMsg cMsg )
