-- Press a button to send a GET request for random quotes.
--
-- Read how it works:
--   https://guide.elm-lang.org/effects/json.html
--


module Main exposing (Model(..), Msg(..), Redirection, fetchRedirections, init, main, redirectionDecoder, redirectionsTable, subscriptions, update, view)

import Browser
import Html exposing (..)
import Html.Attributes exposing (href, style)
import Html.Events exposing (..)
import Http
import Json.Decode exposing (Decoder, field, int, list, map3, string)



-- MAIN


main =
    Browser.element
        { init = init
        , update = update
        , subscriptions = subscriptions
        , view = view
        }



-- MODEL


type Model
    = Failure
    | Loading
    | Success (List Redirection)


type alias Redirection =
    { long_url : String
    , short_url : String
    , id : Int
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Loading, fetchRedirections )



-- UPDATE


type Msg
    = MorePlease
    | GotRedirections (Result Http.Error (List Redirection))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        MorePlease ->
            ( Loading, fetchRedirections )

        GotRedirections result ->
            case result of
                Ok redirections ->
                    ( Success redirections, Cmd.none )

                Err _ ->
                    ( Failure, Cmd.none )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- VIEW


view : Model -> Html Msg
view model =
    div []
        [ h2 [] [ text "Rus" ]
        , redirectionsTable model
        ]


redirectionsTable : Model -> Html Msg
redirectionsTable model =
    case model of
        Failure ->
            div []
                [ text "Failed to load the shortened links. "
                , button [ onClick MorePlease ] [ text "Try Again!" ]
                ]

        Loading ->
            text "Loading..."

        Success redirections ->
            div []
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
                , button [ onClick MorePlease, style "display" "block" ] [ text "Shorten url" ]
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
        ]



-- HTTP


fetchRedirections : Cmd Msg
fetchRedirections =
    Http.get
        { url = "/api/v1/redirections"
        , expect = Http.expectJson GotRedirections redirectionDecoder
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
