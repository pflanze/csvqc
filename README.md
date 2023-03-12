# CSV quality checker

This is the beginnings or a skeleton or framework for a verification
tool for CSV files. The aim is for it to be fast enough to verify
files as they are being uploaded by users to a server. It currently
handles about a saturated 1 gbit/s network connection on a reasonably
fast server, while using a single CPU core. Of course the speed
depends on how costly the file verifications need to be.

The program is written in Rust, and currently requires the checks to
be written in that language.

It reports back a stream of check failures (which could of course be
cut short, tallied, etc.).

It has a streaming architecture, i.e. is holding only one row of the
CSV file in memory at a time (as long as no verificators retain memory
e.g. to cross-verify cells), and thus runs in constant memory.

Currently this is a proof of concept / early version that runs on the
command line only.

## Possible future work

* Verification of the header row.

* Auto-detection of the line encoding (CR/LF/CRLF), separators (tab
  etc.), and charset encoding (UTF-8 etc.) -- try parsing multiple
  ways (in parallel on multiple cores) in case of ambiguities.

* Transforming the file during upload, e.g.:

    * recode into a canonical format instead of rejecting the file
    * remove trailing whitespace from cells

* Implement upload, by either embedding an HTTP server or implementing
  FastCGI, CGI, or other server integration protocol, and adding a
  simple HTML/JavaScript web app for the upload that can show the back
  stream of check failures while still uploading (to enable a manual
  decision to cancelling while still uploading).

* If interesting for users, allow client side verification in the
  user's browser without upload, via WASM.

* Allow the configuration of file verifications without having to
  write Rust code (using atomic checkers predefined in Rust).

* Prevent resource exhaustion attacks: 

    * make sure that overly long CSV rows and cells are handled
      gracefully
    * implement file size, number of files per second, number of files
      per IP limits

* Performance work (if warranted):

    * Profile and optimize. The `dyn` feature and boxed closures are
      currently used for flexibility, these may be candidates for removal
      if not needed.
    * More performance, if necessary, could also be gained using
      parallelism, by dispatching the incoming stream in chunks to
      multiple threads.

## License & Contact

This code is © Christian Jaeger <ch@christianjaeger.ch> and licensed
under the GNU Public License version 3. Please contact me for other
licenses or further work.
