module Main where

import Lib
import Options.Applicative
import Data.Maybe
import Text.Regex.TDFA
import Data.Monoid
import Data.List (replicate)
import Data.Array (elems)
import System.IO (isEOF)
import qualified Data.Sequence as SQ
import Data.Foldable (toList, minimum, maximum)
import System.Console.ANSI

defaultRadius :: Int
defaultRadius = 5

proceed :: CLOps -> IO ()
proceed clops = processLine clops False SQ.empty

areMatches :: CLOps -> SQ.Seq String -> Bool
areMatches clops lines = 
    getAll $ mconcat $ All <$> [ 
        getAny $ mconcat $ Any <$> [ 
            ln =~ rx 
            | ln <- toList lines 
            ] 
        | rx <- rxs clops 
        ]

allMatchPositions :: CLOps -> String -> [MatchArray]
allMatchPositions clops ln = map snd $ filter fst [ (ln =~ rx, ln =~ rx) | rx <- rxs clops ]

allStarts :: [MatchArray] -> [Int]
allStarts mas = concat [ map fst $ elems ma | ma <- mas ]

allEnds :: [MatchArray] -> [Int]
allEnds mas = concat [ [ fst m + snd m | m <- elems ma]  | ma <- mas ]

printColoredLine :: CLOps -> [(Int, Char)] -> [Int] -> [Int] -> IO ()
printColoredLine clops [] _ _ = putStrLn ""
printColoredLine clops ((i,c):ics) starts ends = do 
    if elem i starts then setSGR [SetColor Foreground Vivid Green] else return ()
    if elem i ends then setSGR [Reset] else return ()
    putChar c
    printColoredLine clops ics starts ends

maxDistanceBetweenMatches :: CLOps -> SQ.Seq String -> Integer
maxDistanceBetweenMatches clops lines =
    let matchess = [ map fst $ filter snd [(i, ln =~ rx) | (i, ln) <- zip [0..] (toList lines) ] | rx <- rxs clops ]
        f = \ m1 m2 -> minimum $ concat [[ abs (i - j) | j <- m2 ] | i <- m1]
    in maximum $ concat [[ f m1 m2 | m1 <- matchess ] | m2 <- matchess ]

processLine :: CLOps -> Bool -> SQ.Seq String -> IO ()
processLine clops printing buf = do 
-- The buffer 'buf' grows up to size 2r+1 
  end <- isEOF
  if end then -- we have reached the end of input
      if areMatches clops buf then 
          if maxDistanceBetweenMatches clops buf < (toInteger $ radius clops)
          then do 
              -- we print r+1 last lines of the buffer, or less if buffer is shorter than r+1
              sequence_ [
                -- putStr "▓▓▓▓▓▓▓▓▓" >> if nocolor clops 
                if nocolor clops
                    then putStrLn ln 
                    else let mas = allMatchPositions clops ln in 
                        printColoredLine clops (zip [0..] ln) (allStarts mas) (allEnds mas)
                | ln <- drop (maximum [0 , length buf - radius clops]) $ toList buf
                ] 
          else return ()
      else return ()
  else do 
    ln <- getLine
    let newbuf = if length buf > 2 * radius clops then SQ.drop 1 buf else buf
    if (length buf >= radius clops) then
        if areMatches clops buf then 
            if maxDistanceBetweenMatches clops buf < (toInteger $ radius clops)
                then do
                    if printing then return () else putStrLn $ replicate 32 '▔'
                    let midln = SQ.index buf $ length buf - radius clops
                    let mas = allMatchPositions clops midln
                    if nocolor clops 
                        then putStrLn midln 
                        else printColoredLine clops (zip [0..] midln) (allStarts mas) (allEnds mas)
                    processLine clops True $ newbuf SQ.|> ln
                else do
                    if printing then putStrLn $ replicate 32 '▁' else return ()
                    processLine clops False $ newbuf SQ.|> ln
        else do 
            if printing then putStrLn $ replicate 32 '▁' else return ()
            processLine clops False $ newbuf SQ.|> ln
    else processLine clops printing (buf SQ.|> ln)

data CLOps = CLOps {
    radius :: Int
    , nocolor :: Bool
    , rxs :: [String] -- ^ regexes
}
cloparser :: Parser CLOps
cloparser = CLOps 
    <$> option auto ( long "radius" <> short 'r' <> metavar "RADIUS" <> help "Radius" <> value defaultRadius)
    <*> switch (long "no-color" <> help "Turn off color highlighting")
    <*> many (argument str (metavar "REGEX..."))
opts :: ParserInfo CLOps
opts = info (cloparser <**> helper)
  ( fullDesc
  <> progDesc "Print matching lines. The text should be sent on STDIN."
  <> header "plurigrep - find groups of neighboring lines in text on STDIN, in which matches occur for all REGEXes from a given set" )

main :: IO ()
main = proceed =<< execParser opts
