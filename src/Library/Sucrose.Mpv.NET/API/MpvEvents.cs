namespace Sucrose.Mpv.NET.API
{
    public partial class Mpv
    {
        public event EventHandler Shutdown;
        public event EventHandler<MpvLogMessageEventArgs> LogMessage;
        public event EventHandler<MpvGetPropertyReplyEventArgs> GetPropertyReply;
        public event EventHandler<MpvSetPropertyReplyEventArgs> SetPropertyReply;
        public event EventHandler<MpvCommandReplyEventArgs> CommandReply;
        public event EventHandler<MpvStartFileEventArgs> StartFile;
        public event EventHandler<MpvEndFileEventArgs> EndFile;
        public event EventHandler<MpvEventHookEventArgs> EventHook;
        public event EventHandler FileLoaded;

        [Obsolete("Deprecated in favour of ObserveProperty on \"track-list\".")]
        public event EventHandler TracksChanged;

        [Obsolete("Deprecated in favour of ObserveProperty on \"vid\", \"aid\" and \"sid\".")]
        public event EventHandler TrackSwitched;

        public event EventHandler Idle;

        [Obsolete("Deprecated in favour of ObserveProperty on \"pause\".")]
        public event EventHandler Pause;

        [Obsolete("Deprecated in favour of ObserveProperty on \"pause\".")]
        public event EventHandler Unpause;

        public event EventHandler Tick;

        [Obsolete("This event does not occur anymore, included for compatibility.")]
        public event EventHandler ScriptInputDispatch;

        public event EventHandler<MpvClientMessageEventArgs> ClientMessage;
        public event EventHandler VideoReconfig;
        public event EventHandler AudioReconfig;

        [Obsolete("Deprecated in favour of ObserveProperty on \"metadata\".")]
        public event EventHandler MetadataUpdate;

        public event EventHandler Seek;
        public event EventHandler PlaybackRestart;
        public event EventHandler<MpvPropertyChangeEventArgs> PropertyChange;

        [Obsolete("Deprecated in favour of ObserveProperty on \"chapter\".")]
        public event EventHandler ChapterChange;

        public event EventHandler QueueOverflow;

        private void EventCallback(MpvEvent @event)
        {
            MpvEventID eventId = @event.Id;
            switch (eventId)
            {
                // Events that can be "handled"
                case MpvEventID.Shutdown:
                    HandleShutdown();
                    break;
                case MpvEventID.LogMessage:
                    HandleLogMessage(@event);
                    break;
                case MpvEventID.GetPropertyReply:
                    HandleGetPropertyReply(@event);
                    break;
                case MpvEventID.SetPropertyReply:
                    HandleSetPropertyReply(@event);
                    break;
                case MpvEventID.CommandReply:
                    HandleCommandReply(@event);
                    break;
                case MpvEventID.StartFile:
                    HandleStartFile(@event);
                    break;
                case MpvEventID.EndFile:
                    HandleEndFile(@event);
                    break;
                case MpvEventID.ClientMessage:
                    HandleClientMessage(@event);
                    break;
                case MpvEventID.PropertyChange:
                    HandlePropertyChange(@event);
                    break;
                case MpvEventID.EventHook:
                    HandleEventHook(@event);
                    break;

                // Todo: Find a better/shorter way of doing this?
                // I tried to put the EventHandlers below into a dictionary
                // and invoke them based on the event ID but a reference
                // to the EventHandler didn't seem to update when a handler
                // was attached to that property and therefore we couldn't invoke
                // it.

                // All other simple notification events.
                case MpvEventID.FileLoaded:
                    InvokeSimple(FileLoaded);
                    break;
                case MpvEventID.TracksChanged:
                    InvokeSimple(TracksChanged);
                    break;
                case MpvEventID.TrackSwitched:
                    InvokeSimple(TrackSwitched);
                    break;
                case MpvEventID.Pause:
                    InvokeSimple(Pause);
                    break;
                case MpvEventID.Unpause:
                    InvokeSimple(Unpause);
                    break;
                case MpvEventID.ScriptInputDispatch:
                    InvokeSimple(ScriptInputDispatch);
                    break;
                case MpvEventID.VideoReconfig:
                    InvokeSimple(VideoReconfig);
                    break;
                case MpvEventID.AudioReconfig:
                    InvokeSimple(AudioReconfig);
                    break;
                case MpvEventID.MetadataUpdate:
                    InvokeSimple(MetadataUpdate);
                    break;
                case MpvEventID.Seek:
                    InvokeSimple(Seek);
                    break;
                case MpvEventID.PlaybackRestart:
                    InvokeSimple(PlaybackRestart);
                    break;
                case MpvEventID.ChapterChange:
                    InvokeSimple(ChapterChange);
                    break;
                case MpvEventID.QueueOverflow:
                    InvokeSimple(QueueOverflow);
                    break;
            }
        }

        private void HandleShutdown()
        {
            eventLoop.Stop();
            Shutdown?.Invoke(this, EventArgs.Empty);
        }

        private void HandleLogMessage(MpvEvent @event)
        {
            if (LogMessage == null)
            {
                return;
            }

            MpvLogMessage? logMessage = @event.MarshalDataToStruct<MpvLogMessage>();
            if (logMessage.HasValue)
            {
                MpvLogMessageEventArgs eventArgs = new(logMessage.Value);
                LogMessage.Invoke(this, eventArgs);
            }
        }

        private void HandleGetPropertyReply(MpvEvent @event)
        {
            if (GetPropertyReply == null)
            {
                return;
            }

            MpvEventProperty? eventProperty = @event.MarshalDataToStruct<MpvEventProperty>();
            if (eventProperty.HasValue)
            {
                ulong replyUserData = @event.ReplyUserData;
                MpvError error = @event.Error;

                MpvGetPropertyReplyEventArgs eventArgs = new(replyUserData, error, eventProperty.Value);
                GetPropertyReply.Invoke(this, eventArgs);
            }
        }

        private void HandleSetPropertyReply(MpvEvent @event)
        {
            if (SetPropertyReply == null)
            {
                return;
            }

            ulong replyUserData = @event.ReplyUserData;
            MpvError error = @event.Error;

            MpvSetPropertyReplyEventArgs eventArgs = new(replyUserData, error);
            SetPropertyReply.Invoke(this, eventArgs);
        }

        private void HandleCommandReply(MpvEvent @event)
        {
            if (CommandReply == null)
            {
                return;
            }

            ulong replyUserData = @event.ReplyUserData;
            MpvError error = @event.Error;

            MpvCommandReplyEventArgs eventArgs = new(replyUserData, error);
            CommandReply.Invoke(this, eventArgs);
        }

        private void HandleStartFile(MpvEvent @event)
        {
            if (StartFile == null)
            {
                return;
            }

            MpvEventStartFile? eventStartFile = @event.MarshalDataToStruct<MpvEventStartFile>();
            if (eventStartFile.HasValue)
            {
                MpvStartFileEventArgs eventArgs = new(eventStartFile.Value);
                StartFile.Invoke(this, eventArgs);
            }
        }

        private void HandleEndFile(MpvEvent @event)
        {
            if (EndFile == null)
            {
                return;
            }

            MpvEventEndFile? eventEndFile = @event.MarshalDataToStruct<MpvEventEndFile>();
            if (eventEndFile.HasValue)
            {
                MpvEndFileEventArgs eventArgs = new(eventEndFile.Value);
                EndFile.Invoke(this, eventArgs);
            }
        }

        private void HandleClientMessage(MpvEvent @event)
        {
            if (ClientMessage == null)
            {
                return;
            }

            MpvEventClientMessage? eventClientMessage = @event.MarshalDataToStruct<MpvEventClientMessage>();
            if (eventClientMessage.HasValue)
            {
                MpvClientMessageEventArgs eventArgs = new(eventClientMessage.Value);
                ClientMessage.Invoke(this, eventArgs);
            }
        }

        private void HandlePropertyChange(MpvEvent @event)
        {
            if (PropertyChange == null)
            {
                return;
            }

            MpvEventProperty? eventProperty = @event.MarshalDataToStruct<MpvEventProperty>();
            if (eventProperty.HasValue)
            {
                ulong replyUserData = @event.ReplyUserData;

                MpvPropertyChangeEventArgs eventArgs = new(replyUserData, eventProperty.Value);
                PropertyChange.Invoke(this, eventArgs);
            }
        }

        private void HandleEventHook(MpvEvent @event)
        {
            if (EventHook == null)
            {
                return;
            }

            MpvEventHook? eventHook = @event.MarshalDataToStruct<MpvEventHook>();
            if (eventHook.HasValue)
            {
                MpvEventHookEventArgs eventArgs = new(eventHook.Value);
                EventHook.Invoke(this, eventArgs);
            }
        }

        private void InvokeSimple(EventHandler eventHandler)
        {
            eventHandler?.Invoke(this, EventArgs.Empty);
        }
    }
}