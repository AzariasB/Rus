module Page exposing (Model, Msg(..), init, update, view)

import Browser exposing (Document)
import Browser.Navigation as Nav
import Html exposing (Html, a, div, footer, h1, header, text)
import Html.Attributes exposing (class, href)
import Pages.Create as C
import Pages.Home as H
import Session
import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, oneOf, s, top)


type alias Model =
    { page : Page
    , session : Session.Data
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
        session =
            Session.default url key

        ( homeModel, _ ) =
            H.init session
    in
    route url { page = Home homeModel, session = session }



-- VIEW


view : Model -> Document Msg
view model =
    case model.page of
        Home subModel ->
            { title = "Home"
            , body = viewHeader :: viewFlash model.session.flash :: Html.map GotHomeMsg (H.view subModel) :: [ viewFooter ]
            }

        Create subModel ->
            { title = "Create"
            , body = viewHeader :: viewFlash model.session.flash :: Html.map GotCreateMsg (C.view subModel) :: [ viewFooter ]
            }


viewHeader : Html msg
viewHeader =
    header []
        [ div [ class "container" ]
            [ h1 [] [ text "Rus" ]
            ]
        ]


viewFlash : Maybe String -> Html msg
viewFlash flash =
    case flash of
        Just message ->
            div
                [ class "container" ]
                [ text message
                ]

        Nothing ->
            div [] []


viewFooter : Html msg
viewFooter =
    footer []
        [ div [ class "container" ]
            [ a [ class "logo-font", href "/" ] [ text "Home" ] ]
        ]



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg ({ page, session } as model) =
    case ( msg, page ) of
        ( GotHomeMsg homeMsg, Home h ) ->
            H.update homeMsg h |> updateWith model Home GotHomeMsg

        ( GotCreateMsg createMsg, Create c ) ->
            case createMsg of
                (C.Internal _) as internal ->
                    C.update internal c |> updateWith model Create GotCreateMsg

                C.External ext ->
                    case ext of
                        C.CreatedLink result ->
                            case result of
                                Ok res ->
                                    if res.error then
                                        ( { model | session = Session.setFlash session res.message }, Cmd.none )

                                    else
                                        ( model, Nav.pushUrl model.session.nav "/" )

                                Err _ ->
                                    let
                                        flashMsg =
                                            { model | session = Session.setFlash model.session "Http error" }
                                    in
                                    ( flashMsg, Cmd.none )

        ( ClickedLink req, _ ) ->
            case req of
                Browser.External href ->
                    ( model, Nav.load href )

                Browser.Internal url ->
                    ( { model | session = Session.removeFlash session }, Nav.pushUrl session.nav (Url.toString url) )

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
    case Parser.parse parser url of
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
            H.init model.session
    in
    ( { model | page = Home hModel }, Cmd.map GotHomeMsg hMsg )


createRoute : Model -> ( Model, Cmd Msg )
createRoute model =
    let
        ( cModel, cMsg ) =
            C.init model.session
    in
    ( { model | page = Create cModel }, Cmd.map GotCreateMsg cMsg )
