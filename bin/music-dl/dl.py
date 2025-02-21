#!/usr/bin/env python3
# AlbumDownloader - A Python script / tool that downloads complete albums via yt-dlp with the right metadata from the Deezer API
# Usage: ./dl.py "Ordinary Songs 3"
# Usage: ./dl.py "Ordinary Songs 3" --list
# Usage: ./dl.py "Ordinary Songs 3" -o ".." --list --cover
# Usage: ./dl.py "Snails House" --artist

import argparse, json, math, mutagen.mp4, os, subprocess, tempfile, sys, time, threading, urllib.parse, urllib.request, re

TRACK_DURATION_SLACK = 5

def escapePath(path):
    path = path.replace('<', '_').replace('>', '_').replace(':', '_').replace('\"', '_').replace('/', '_')
    return path.replace('\\', '_').replace('|', '_').replace('?', '_').replace('*', '_')

size = None
cursorY = 0
printLock = threading.Lock()
def printLine(y, line):
    global cursorY
    printLock.acquire()
    try:
        if (y <= cursorY):
            sys.stdout.write('\033[F' * (cursorY - y))
        if (y > cursorY):
            sys.stdout.write('\033[E' * (y - cursorY))
        sys.stdout.write('\033[K' + line + '\n')
        cursorY = y + 1
    finally:
        printLock.release()

def cut(string, width):
    if len(string) > width - 4:
        return string[:width - 4] + '... '
    return string

album = None
folderPath = None
coverFilePath = None
def downloadThread(index, track, searchAttempt, searchQuery):
    track = json.load(urllib.request.urlopen('https://api.deezer.com/track/' + str(track['id'])))

    # Search video
    with subprocess.Popen(['yt-dlp', '--dump-json', 'ytsearch25:' + searchQuery], stdout=subprocess.PIPE) as searchProcess:
        while searchProcess.poll() is None:
            line = searchProcess.stdout.readline().decode()
            try:
                videoJson = json.loads(line)
                if track['duration'] >= videoJson['duration'] - TRACK_DURATION_SLACK and track['duration'] <= videoJson['duration'] + TRACK_DURATION_SLACK:
                    searchProcess.terminate()

                    # Download video audio
                    path = folderPath + '/' + escapePath(album['artist']['name'] + ' - ' + album['title'] + ' - ' + (('%0' + str(len(str(album['nb_tracks']))) + 'd') % index) + ' - ' + track['title'] + '.m4a')
                    with subprocess.Popen(['yt-dlp', '--newline', '-f', 'bestaudio[ext=m4a]', 'https://www.youtube.com/watch?v=' + videoJson['id'], '-o', path], stdout=subprocess.PIPE) as downloadProcess:
                        while downloadProcess.poll() is None:
                            line = downloadProcess.stdout.readline().decode()
                            procents = re.findall(r'[\d\.]+%', line)
                            if len(procents) > 0:
                                procent = float(procents[0][:-1]) / 100
                                width = math.floor(size.columns * 0.5) - 2
                                filled = math.ceil(width * procent)
                                leftColumn = cut(('%0' + str(len(str(album['nb_tracks']))) + 'd. %s (%d:%02d)') % (index, track['title'], track['duration'] / 60, track['duration'] % 60), math.floor(size.columns * 0.35))
                                middleColumn = cut('Downloading video...', math.floor(size.columns * 0.15))
                                printLine(index, '%s%s%s%s[%s%s]' % (
                                    leftColumn, ' ' * (math.floor(size.columns * 0.35) - len(leftColumn)),
                                    middleColumn, ' ' * (math.floor(size.columns * 0.15) - len(middleColumn)),
                                    '#' * filled, '-' * (width - filled)
                                ))

                    # Correct video metadata
                    file = mutagen.mp4.MP4(path)
                    file['\xa9nam'] = track['title']
                    file['\xa9alb'] = album['title']
                    albumArtists = [ artist['name'] for artist in album['contributors'] ]
                    file['\xa9ART'] = ', '.join(albumArtists + [ artist['name'] for artist in track['contributors'] if artist['name'] not in albumArtists ])
                    file['aART'] = ', '.join(albumArtists)
                    file['\xa9day'] = album['release_date'].split('-')[0]
                    file['trkn'] = [ (index, album['nb_tracks']) ]
                    file['\xa9gen'] = ', '.join([ genre['name'] for genre in album['genres']['data'] ])
                    with open(coverFilePath, 'rb') as coverFile:
                        file['covr'] = [ mutagen.mp4.MP4Cover(coverFile.read(), imageformat=mutagen.mp4.MP4Cover.FORMAT_JPEG) ]
                    file.save()

                    leftColumn = cut(('%0' + str(len(str(album['nb_tracks']))) + 'd. %s (%d:%02d)') % (index, track['title'], track['duration'] / 60, track['duration'] % 60), math.floor(size.columns * 0.35))
                    middleColumn = cut('Done', math.floor(size.columns * 0.15))
                    printLine(index, '%s%s%s%s[%s]' % (
                        leftColumn, ' ' * (math.floor(size.columns * 0.35) - len(leftColumn)),
                        middleColumn, ' ' * (math.floor(size.columns * 0.15) - len(middleColumn)),
                        '#' * (math.floor(size.columns * 0.5) - 2)
                    ))
                    break
            except:
                searchProcess.terminate()

                # Try another search query or fail
                if searchAttempt == 1:
                    downloadThread(index, track, 2, album['artist']['name'] + ' - ' + album['title'] + ' - ' + track['title'])
                elif searchAttempt == 2:
                    downloadThread(index, track, 3, album['title'] + ' - ' + track['title'])
                else:
                    leftColumn = cut(('%0' + str(len(str(album['nb_tracks']))) + 'd. %s (%d:%02d)') % (index, track['title'], track['duration'] / 60, track['duration'] % 60), math.floor(size.columns * 0.35))
                    middleColumn = cut('Can\'t find video', math.floor(size.columns * 0.15))
                    printLine(index, '%s%s%s%s[%s]' % (
                        leftColumn, ' ' * (math.floor(size.columns * 0.35) - len(leftColumn)),
                        middleColumn, ' ' * (math.floor(size.columns * 0.15) - len(middleColumn)),
                        '-' * (math.floor(size.columns * 0.5) - 2)
                    ))
                return

args = None
def handleAlbum(albumId):
    global album, folderPath, coverFilePath, size

    # Query album information and list when needed
    album = json.load(urllib.request.urlopen('https://api.deezer.com/album/' + str(albumId)))
    albumArtists = [ artist['name'] for artist in album['contributors'] ]
    if args.list:
        print('# ' + album['title'] + ' by ' + ', '.join(albumArtists))
        print('Released at ' + album['release_date'] + ' with ' + str(album['nb_tracks']) + ' tracks')
        index = 1
        for track in album['tracks']['data']:
            track = json.load(urllib.request.urlopen('https://api.deezer.com/track/' + str(track['id'])))
            trackArtists = ', '.join(albumArtists + [ artist['name'] for artist in track['contributors'] if artist['name'] not in albumArtists ])
            print('%d. %s (%d:%02d) by %s' % (index, track['title'], track['duration'] / 60, track['duration'] % 60, trackArtists))
            index += 1
        return

    # Create album download folder
    folderPath = args.output + '/' + escapePath(album['artist']['name']) + '/' + escapePath(album['title'])
    if os.path.isdir(folderPath):
        print('Skipping ' + album['title'] + ' by ' + ', '.join(albumArtists) + ' it already exists...')
        return
    os.makedirs(folderPath, exist_ok=True)

    # Download album cover
    if args.cover:
        coverFilePath = folderPath + '/cover.jpg'
    else:
        _, coverFilePath = tempfile.mkstemp()
    urllib.request.urlretrieve(album['cover_xl'], coverFilePath)

    # Create download threads
    size = os.get_terminal_size()
    printLine(0, '# ' + album['title'] + ' by ' + ', '.join(albumArtists))
    index = 1
    threads = []
    for track in album['tracks']['data']:
        leftColumn = cut(('%0' + str(len(str(album['nb_tracks']))) + 'd. %s (%d:%02d)') % (index, track['title'], track['duration'] / 60, track['duration'] % 60), math.floor(size.columns * 0.35))
        middleColumn = cut('Searching video...', math.floor(size.columns * 0.15))
        printLine(index, '%s%s%s%s[%s]' % (
            leftColumn, ' ' * (math.floor(size.columns * 0.35) - len(leftColumn)),
            middleColumn, ' ' * (math.floor(size.columns * 0.15) - len(middleColumn)),
            '-' * (math.floor(size.columns * 0.5) - 2)
        ))
        threads.append(threading.Thread(target=downloadThread, args=[index, track, 1, album['artist']['name'] + ' - ' + track['title']]))
        index += 1

    # Start and wait for all threads to finish
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()

    # Clear screen
    for i in range(index):
        printLine(i, '')
    printLine(0, '')

def main():
    global args

    # Parse arguments
    parser = argparse.ArgumentParser(description='A Python script / tool that downloads complete albums via the YouTube API and yt-dlp with the right metadata via the Deezer API')
    parser.add_argument('query')
    parser.add_argument('-o', '--output', default=os.path.expanduser('~') + '/Music', help='The output directory')
    parser.add_argument('-l', '--list', action='store_true', help='Just list the music metadata')
    parser.add_argument('-c', '--cover', action='store_true', help='Save album cover as a seperated file')
    parser.add_argument('-a', '--artist', action='store_true', help='Search query is an artist and download all its albums and EP\'s')
    parser.add_argument('-s', '--singles', action='store_true', help='Download also singles when downloading all artists stuff')
    parser.add_argument('-i', '--id', action='store_true', help='Search query is an Deezer Id don\'t search album or artist')
    args = parser.parse_args()

    # Search for artist with Deezer API
    if args.artist:
        artists = None
        if not args.id:
            artists = json.load(urllib.request.urlopen('https://api.deezer.com/search/artist?q=' + urllib.parse.quote_plus(args.query)))['data']
            if len(artists) == 0:
                print('No artist found!')
                return

        # Handle artists albums
        albums = json.load(urllib.request.urlopen('https://api.deezer.com/artist/' + (artists != None and str(artists[0]['id']) or args.query) + '/albums'))
        for album in albums['data']:
            if args.singles or (album['type'] in ['album', 'ep'] and album['record_type'] != 'single'):
                handleAlbum(album['id'])
                if args.list:
                    print()
                    time.sleep(1)
        return

    # Search for album with Deezer API
    albums = None
    if not args.id:
        albums = json.load(urllib.request.urlopen('https://api.deezer.com/search/album?q=' + urllib.parse.quote_plus(args.query)))['data']
        if len(albums) == 0:
            print('No album found!')
            return

    # Handle found album
    handleAlbum(albums != None and albums[0]['id'] or args.query)

if __name__ == '__main__':
    main()
