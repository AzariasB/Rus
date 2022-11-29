module Page exposing (Model, Msg(..), init, update, view)

import Browser exposing (Document)
import Browser.Navigation as Nav
import Convert exposing (httpErrorToString)
import Html exposing (Html, a, div, footer, h1, header, text)
import Html.Attributes exposing (class, href)
import Pages.Edit as C
import Pages.Error as E
import Pages.Home as H
import Redirection exposing (Redirection)
import Session
import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, oneOf, s, string, top)


type alias Model =
    { page : Page
    , session : Session.Data
    }


type Page
    = Home H.Model
    | Edit C.Model
    | Error E.Model


type Msg
    = ChangedUrl Url
    | ClickedLink Browser.UrlRequest
    | GotHomeMsg H.Msg
    | GotEditMsg C.Msg
    | GotErrorMsg E.Msg


init : Url -> Nav.Key -> ( Model, Cmd Msg )
init url key =
    let
        session =
            Session.default url key

        ( homeModel, _ ) =
            H.init session
    in
    preRouting url { page = Home homeModel, session = session }



-- VIEW


view : Model -> Document Msg
view model =
    let
        displayPage title htmlMap pageView =
            { title = title, body = viewHeader :: viewFlash model.session.flash :: Html.map htmlMap pageView :: [ viewFooter ] }
    in
    case model.page of
        Home subModel ->
            displayPage "Home" GotHomeMsg <| H.view subModel

        Edit subModel ->
            displayPage "Create" GotEditMsg <| C.view subModel

        Error subModel ->
            displayPage "Error" GotErrorMsg <| E.view subModel


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
        ( GotHomeMsg homeMsg, Home homeModel ) ->
            case homeMsg of
                H.External external ->
                    case external of
                        H.GotRedirections result ->
                            case result of
                                Ok redirections ->
                                    H.update homeMsg homeModel
                                        |> updateWith
                                            { model
                                                | session = Session.setRedirections model.session redirections
                                            }
                                            Home
                                            GotHomeMsg

                                _ ->
                                    H.update homeMsg homeModel |> updateWith model Home GotHomeMsg

                        H.DeletedRedirection result ->
                            case result of
                                Ok confirmation ->
                                    if confirmation.error then
                                        H.update homeMsg homeModel
                                            |> updateWith
                                                { model
                                                    | session = Session.setFlash model.session confirmation.message
                                                }
                                                Home
                                                GotHomeMsg

                                    else
                                        let
                                            nwSession =
                                                Session.mapRedirections (Session.setFlash model.session "Redirection deleted") <|
                                                    removeRedirectionById confirmation.id
                                        in
                                        H.update homeMsg homeModel
                                            |> updateWith { model | session = nwSession }
                                                Home
                                                GotHomeMsg

                                Err err ->
                                    H.update homeMsg homeModel
                                        |> updateWith
                                            { model
                                                | session = Session.setFlash model.session ("Http Error. " ++ httpErrorToString err)
                                            }
                                            Home
                                            GotHomeMsg

                _ ->
                    H.update homeMsg homeModel |> updateWith model Home GotHomeMsg

        ( GotEditMsg editMsg, Edit editModel ) ->
            case editMsg of
                (C.Internal _) as internal ->
                    C.update internal editModel |> updateWith model Edit GotEditMsg

                C.External ext ->
                    case ext of
                        C.EditJson result ->
                            case result of
                                Ok res ->
                                    if res.error then
                                        ( { model | session = Session.setFlash session res.message }, Cmd.none )

                                    else
                                        let
                                            nwSession =
                                                Session.withoutRedirections <| Session.setFlash session "Saved"
                                        in
                                        ( { model | session = nwSession }, Nav.pushUrl model.session.nav "/" )

                                Err err ->
                                    let
                                        flashMsg =
                                            { model | session = Session.setFlash model.session <| "Http error : " ++ httpErrorToString err }
                                    in
                                    ( flashMsg, Cmd.none )

                        _ ->
                            ( model, Cmd.none )

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


removeRedirectionById : Int -> List Redirection -> List Redirection
removeRedirectionById id list =
    List.filter (\r -> id /= r.id) list


preRouting : Url -> Model -> ( Model, Cmd Msg )
preRouting url model =
    case Parser.parse (routeParser model) url of
        Just answer ->
            answer

        Nothing ->
            updateWith model Error GotErrorMsg (E.init "Page not found")


route : Url -> Model -> ( Model, Cmd Msg )
route url model =
    case Parser.parse (routeParser model) url of
        Just answer ->
            answer

        Nothing ->
            ( model, Nav.load (Url.toString url) )


routeParser : Model -> Parser (( Model, Cmd Msg ) -> a) a
routeParser model =
    oneOf
        [ pageRoute top (homeRoute model)
        , pageRoute (s "create") (createRoute model)
        , pageRoute (s "edit" </> string) (editRoute model)
        ]


pageRoute : Parser a b -> a -> Parser (b -> c) c
pageRoute parser handler =
    Parser.map handler parser


homeRoute : Model -> ( Model, Cmd Msg )
homeRoute model =
    updateWith model Home GotHomeMsg (H.init model.session)


createRoute : Model -> ( Model, Cmd Msg )
createRoute model =
    updateWith model Edit GotEditMsg (C.init model.session ( "", C.createLink ))


editRoute : Model -> String -> ( Model, Cmd Msg )
editRoute model short_url =
    let
        redirection =
            Session.findRedirection model.session short_url
    in
    case redirection of
        Nothing ->
            homeRoute model

        Just red ->
            updateWith model Edit GotEditMsg (C.init model.session ( red.long_url, C.editLink red.short_url ))


updateWith : Model -> (subModel -> Page) -> (subMsg -> Msg) -> ( subModel, Cmd subMsg ) -> ( Model, Cmd Msg )
updateWith model toModel toMsg ( subModel, subCmd ) =
    ( { model | page = toModel subModel }, Cmd.map toMsg subCmd )
