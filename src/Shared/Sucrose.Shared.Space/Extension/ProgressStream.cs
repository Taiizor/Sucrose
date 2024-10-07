using System.IO;

namespace Sucrose.Shared.Space.Extension
{
    internal class ProgressStream(Stream Stream, long TotalBytes, Action<long, long, double> Progress) : Stream
    {
        public event EventHandler<EventArgs>? ProgressCompleted;

        public event EventHandler<EventArgs>? ProgressStarted;

        public event EventHandler<Exception>? ProgressFailed;

        public override bool CanWrite => Stream.CanWrite;

        public override bool CanRead => Stream.CanRead;

        public override bool CanSeek => Stream.CanSeek;

        public override long Length => Stream.Length;

        private bool IsProgressComplete = false;

        private bool IsProgressStart = false;

        private long BytesTransferred;

        public override void Flush()
        {
            Stream.Flush();
        }

        public override long Position
        {
            get => Stream.Position;
            set => Stream.Position = value;
        }

        public override void SetLength(long value)
        {
            Stream.SetLength(value);
        }

        protected virtual void OnProgressStarted(EventArgs Args)
        {
            ProgressStarted?.Invoke(this, Args);
        }

        public override long Seek(long offset, SeekOrigin origin)
        {
            return Stream.Seek(offset, origin);
        }

        protected virtual void OnProgressCompleted(EventArgs Args)
        {
            ProgressCompleted?.Invoke(this, Args);
        }

        protected virtual void OnProgressFailed(Exception Exception)
        {
            ProgressFailed?.Invoke(this, Exception);
        }

        public override int Read(byte[] buffer, int offset, int count)
        {
            try
            {
                if (!IsProgressStart)
                {
                    IsProgressStart = true;

                    OnProgressStarted(EventArgs.Empty);
                }

                int bytesRead = Stream.Read(buffer, offset, count);

                BytesTransferred += bytesRead;

                Progress(BytesTransferred, TotalBytes, (double)BytesTransferred / TotalBytes * 100);

                if (BytesTransferred >= TotalBytes && !IsProgressComplete)
                {
                    IsProgressComplete = true;

                    OnProgressCompleted(EventArgs.Empty);
                }

                return bytesRead;
            }
            catch (Exception Exception)
            {
                OnProgressFailed(Exception);

                throw;
            }
        }

        public override void Write(byte[] buffer, int offset, int count)
        {
            try
            {
                if (!IsProgressStart)
                {
                    IsProgressStart = true;

                    OnProgressStarted(EventArgs.Empty);
                }

                Stream.Write(buffer, offset, count);

                BytesTransferred += count;

                Progress(BytesTransferred, TotalBytes, (double)BytesTransferred / TotalBytes * 100);

                if (BytesTransferred >= TotalBytes && !IsProgressComplete)
                {
                    IsProgressComplete = true;

                    OnProgressCompleted(EventArgs.Empty);
                }
            }
            catch (Exception Exception)
            {
                OnProgressFailed(Exception);

                throw;
            }
        }
    }
}