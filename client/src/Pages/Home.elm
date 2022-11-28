module Pages.Home exposing (ExternalMsg(..), InternalMsg, Model, Msg(..), fetchRedirections, init, update, view)

import Html exposing (..)
import Html.Attributes exposing (class, href)
import Html.Events exposing (..)
import Http
import Json.Decode exposing (Decoder, field, int, list, map3, string)
import Redirection exposing (Redirection)
import Session



-- MODEL


type Status
    = Failure
    | Loading
    | Success (List Redirection)


type alias Model =
    { status : Status, session : Session.Data }


init : Session.Data -> ( Model, Cmd Msg )
init data =
    let
        status =
            case data.redirections of
                Just redirections ->
                    Success redirections

                Nothing ->
                    Loading

        model =
            { status = status, session = data }
    in
    ( model, fetchRedirections model )



-- UPDATE


type InternalMsg
    = Refresh


type ExternalMsg
    = GotRedirections (Result Http.Error (List Redirection))


type Msg
    = Internal InternalMsg
    | External ExternalMsg


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Internal Refresh ->
            ( { model | status = Loading }, fetchRedirections { model | session = Session.withoutRedirections model.session } )

        External (GotRedirections result) ->
            case result of
                Ok redirections ->
                    ( { model | status = Success redirections }, Cmd.none )

                Err _ ->
                    ( { model | status = Failure }, Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    case model.status of
        Failure ->
            div []
                [ text "Failed to load the shortened links. "
                , button [ onClick <| Internal Refresh ] [ text "Try Again!" ]
                ]

        Loading ->
            text "Loading..."

        Success redirections ->
            div [ class "container" ]
                [ table []
                    [ thead
                        []
                        [ tr []
                            [ th [] [ text "ID" ]
                            , th [] [ text "Long url" ]
                            , th [] [ text "Short url" ]
                            , th [] []
                            ]
                        ]
                    , tbody [] (List.map redirectionRow redirections)
                    ]
                , a [ href "/create", class "button" ] [ text "Shorten url" ]
                ]


redirectionRow : Redirection -> Html Msg
redirectionRow red =
    tr []
        [ td [] [ text (String.fromInt red.id) ]
        , td []
            [ a [ href red.long_url ] [ text red.long_url ]
            ]
        , td []
            [ a [ href red.short_url ] [ text red.short_url ]
            ]
        , td []
            [ a [ href ("/edit/" ++ String.fromInt red.id), class "small button" ] [ text "Edit" ]
            , button [ class "small delete-button" ] [ text "Delete" ]
            ]
        ]



-- HTTP


fetchRedirections : Model -> Cmd Msg
fetchRedirections model =
    case model.session.redirections of
        Just _ ->
            Cmd.none

        Nothing ->
            Http.get
                { url = "/api/v1/redirections"
                , expect = Http.expectJson (GotRedirections >> External) redirectionDecoder
                }


redirectionDecoder : Decoder (List Redirection)
redirectionDecoder =
    list
        (map3
            Redirection
            (field "long_url" string)
            (field "short_url" string)
            (field "id" int)
        )
