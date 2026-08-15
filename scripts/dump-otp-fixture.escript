#!/usr/bin/env escript
%%! -noshell -noinput

%% Walk .erl files under a directory and dump erl_scan tokens as JSONL.
%%
%% Each line is a complete JSON object of the form:
%%   {"<path>":[["<kind>","<text>"],["<kind>","<text>"],...]}
%%
%% * `<path>` is the file path (relative to the caller's cwd or absolute,
%%   depending on how the input directory is specified).
%% * `<kind>` is the erl_scan token kind atom (e.g. `atom`, `integer`,
%%   `dot`, or a symbol atom such as `,` or `case`) stringified via
%%   `atom_to_list/1`.
%% * `<text>` is the original source slice recovered via the `text`
%%   option of `erl_scan:string/3`.
%%
%% `erl_scan:string/3` is called with only the `[text]` option. Neither
%% `return_white_spaces` nor `return_comments` is specified, so the
%% resulting token list is lexical only (white_space and comment tokens
%% are excluded from the fixture).
%%
%% Files that erl_scan cannot tokenize (`{error, _, _}` return) are
%% skipped silently; downstream tooling should not expect an entry for
%% them.

main([Dir]) ->
    Files = walk(Dir),
    lists:foreach(fun dump_file/1, Files);
main(_) ->
    io:format(standard_error, "usage: dump.escript <dir>~n", []),
    halt(1).

walk(Dir) ->
    filelib:fold_files(Dir, ".*\\.erl$", true,
                       fun(F, Acc) -> [F | Acc] end, []).

dump_file(F) ->
    case file:read_file(F) of
        {ok, Bin} ->
            %% Decode file as UTF-8 to a list of Unicode codepoints. Feeding
            %% raw bytes (via `binary_to_list/1`) would leave multi-byte
            %% UTF-8 sequences as separate byte values, which io:put_chars
            %% then re-encodes assuming each value is a codepoint — causing
            %% double-encoding mojibake for non-ASCII text.
            case unicode:characters_to_list(Bin, utf8) of
                Src when is_list(Src) ->
                    case erl_scan:string(Src, {1, 1}, [text]) of
                        {ok, Toks, _End} -> emit(F, merge_sigils(Toks));
                        {error, _EI, _EL} -> ok
                    end;
                _ -> ok
            end;
        _ -> ok
    end.

%% erl_scan splits a sigil literal into three tokens:
%% `sigil_prefix` (e.g. `~` or `~b`), `string` (the quoted body), and
%% `sigil_suffix` (usually empty). erl_tokenize returns the whole sigil
%% as a single `SigilString` token. Merge the three back into a single
%% pseudo-token of kind `sigil_string` so the fixture matches
%% erl_tokenize's granularity 1:1.
%% erl_scan returns these three tokens as 3-tuples `{Kind, Anno, Value}`:
%% the sigil_prefix carries the prefix atom (`''` / `b` / `s` / ...),
%% the string carries the decoded body, and the sigil_suffix carries the
%% suffix atom. Only the first two annotations carry text; sigil_suffix's
%% second element is a bare location (its text is undefined).
merge_sigils([{sigil_prefix, PAnno, _}, {string, SAnno, _}, {sigil_suffix, _, _} | Rest]) ->
    Text = anno_text(PAnno) ++ anno_text(SAnno),
    Anno = case PAnno of
               A when is_list(A) -> erl_anno:set_text(Text, A);
               Loc -> erl_anno:set_text(Text, erl_anno:new(Loc))
           end,
    [{sigil_string, Anno} | merge_sigils(Rest)];
merge_sigils([T | Rest]) ->
    [T | merge_sigils(Rest)];
merge_sigils([]) ->
    [].

anno_text(Anno) ->
    case erl_anno:text(Anno) of
        undefined -> "";
        Txt when is_list(Txt) -> Txt;
        Txt when is_binary(Txt) -> binary_to_list(Txt)
    end.

emit(F, Toks) ->
    Body = string:join([token_json(T) || T <- Toks], ","),
    io:put_chars([${, $", escape(F), $", $:, $[, Body, $], $}, $\n]).

token_json(T) ->
    Kind = element(1, T),
    Anno = element(2, T),
    Text = anno_text(Anno),
    %% erl_scan attaches the trailing form-separator whitespace (e.g.
    %% `.\n`) to the `dot` token's text annotation, unlike every other
    %% kind whose text is exactly the source slice of the token itself.
    %% Normalize the dot text to just `"."` so downstream diff tooling
    %% does not have to special-case the whitespace tail.
    Text2 = case Kind of
                dot -> ".";
                _ -> Text
            end,
    [$[, $", atom_to_list(Kind), $", $,, $", escape(Text2), $", $]].

escape(S) when is_list(S) -> lists:flatten([escape_char(C) || C <- S]);
escape(B) when is_binary(B) -> escape(binary_to_list(B)).

escape_char($\\) -> "\\\\";
escape_char($") -> "\\\"";
escape_char($\n) -> "\\n";
escape_char($\r) -> "\\r";
escape_char($\t) -> "\\t";
escape_char($\b) -> "\\b";
escape_char($\f) -> "\\f";
escape_char(C) when C < 32 -> io_lib:format("\\u~4.16.0b", [C]);
escape_char(C) -> C.
