use std::task::Poll;

use anyhow::Result;
use futures::{ready, Stream};
use pin_project_lite::pin_project;
use serde_json::Value;

mod buffer;

pub use buffer::ObjectBuffer;

use crate::{sources::RowSource, view_plan::ViewPlan, ObjectIdPair};

pin_project! {
    pub struct ObjectBufferedStream<S> {
        buffer: ObjectBuffer,
        vec: Vec<RowSource>,
        #[pin]
        input: S,
    }
}

impl<S> ObjectBufferedStream<S>
where
    S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
{
    pub fn new(input: S, view_plan: ViewPlan) -> Self {
        Self {
            buffer: ObjectBuffer::new(view_plan),
            vec: Default::default(),
            input,
        }
    }
}

impl<S> Stream for ObjectBufferedStream<S>
where
    S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
{
    type Item = Result<RowSource>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(row) = this.vec.pop() {
                break Some(Ok(row));
            } else if let Some(s) = ready!(this.input.as_mut().poll_next(cx)) {
                match s {
                    Err(e) => break (Some(Err(e))),
                    Ok((object_pair, object)) => {
                        match this.buffer.add_object(object_pair, object) {
                            None => continue,
                            Some(Err(e)) => {
                                break (Some(Err(e)));
                            }
                            Some(Ok(rows)) => {
                                *this.vec = rows;
                            }
                        }
                    }
                }
            } else {
                break None;
            }
        })
    }
}

#[cfg(all(test, not(miri)))]
mod tests {
    use cdl_dto::materialization::{FieldDefinition, FieldType, FullView};
    use futures::{pin_mut, FutureExt, StreamExt, TryStreamExt};
    use maplit::*;
    use serde_json::json;
    use tokio::sync::mpsc::{channel, Sender};
    use tokio_stream::wrappers::ReceiverStream;
    use uuid::Uuid;

    use super::*;
    use crate::{
        sources::FieldDefinitionSource,
        view_plan::{UnfinishedRow, ViewPlan},
    };

    impl<S> ObjectBufferedStream<S>
    where
        S: Stream<Item = Result<(ObjectIdPair, Value)>> + Unpin,
    {
        pub fn new_test(input: S, plan: ViewPlan) -> Self {
            Self {
                buffer: ObjectBuffer::new(plan),
                vec: Default::default(),
                input,
            }
        }
    }

    fn any_view(object: ObjectIdPair) -> FullView {
        FullView {
            id: Uuid::new_v4(),
            base_schema_id: object.schema_id,
            name: "".into(),
            materializer_address: "".into(),
            materializer_options: json!({}),
            fields: hashmap! {
                "foo".into() => FieldDefinition::Simple { field_name: "foo".into(), field_type: FieldType::String }
            },
            relations: vec![],
            filters: None,
        }
    }

    #[tokio::test]
    async fn when_there_are_no_edges() {
        let obj = new_obj(None);

        let plan = ViewPlan {
            unfinished_rows: Default::default(),
            missing: Default::default(),
            view: any_view(obj.0),
            single_mode: true,
        };

        let (tx, stream) = act(plan);
        pin_mut!(stream);

        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(obj.clone())).await.unwrap();

        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Single {
                root_object: obj.0,
                value: obj.1,
                filters: None,
                fields: hashmap! {
                    "foo".into() => FieldDefinitionSource::Simple {
                        field_name: "foo".into(),
                        field_type: FieldType::String,
                        object: obj.0
                    }
                }
            }
        );
    }

    #[tokio::test]
    async fn when_there_is_collecting() {
        let child_schema = Uuid::new_v4();
        let a = new_obj(None);
        let b = new_obj(child_schema);
        let c = new_obj(child_schema);

        let a_id = a.0;
        let b_id = b.0;
        let c_id = c.0;

        let objects: Vec<_> = vec![a, b, c];
        // Reversed order - to simulate the fact that objects can arrive via network in any order;
        let mut objects_it = objects.clone().into_iter().rev();

        let plan = ViewPlan {
            unfinished_rows: vec![
                Some(UnfinishedRow {
                    fields: hashmap! {
                        "foo".into() => FieldDefinitionSource::Computed {
                            field_type: FieldType::Numeric,
                            computation: crate::sources::ComputationSource::FieldValue {
                                object: b_id,
                                field_path: "".into()
                            }
                        }
                    },
                    filters: None,
                    missing: 2,
                    root_object: a_id,
                    objects: Default::default(),
                    relation_order: vec![a_id],
                }),
                Some(UnfinishedRow {
                    fields: hashmap! {
                        "foo".into() => FieldDefinitionSource::Computed {
                            field_type: FieldType::Numeric,
                            computation: crate::sources::ComputationSource::FieldValue {
                                object: c_id,
                                field_path: "".into()
                            }
                        }
                    },
                    filters: None,
                    missing: 2,
                    root_object: a_id,
                    objects: Default::default(),
                    relation_order: vec![a_id],
                }),
            ],
            missing: hashmap! {
                a_id => vec![0, 1],
                b_id => vec![0],
                c_id => vec![1]
            },
            view: any_view(a_id),
            single_mode: false,
        };

        let (tx, stream) = act(plan);

        let input = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        });

        let objects = tokio::spawn(async move {
            pin_mut!(stream);
            while let Some(object) = stream.try_next().await.unwrap() {
                dbg!(&object);
            }
        });

        let objects = tokio::time::timeout(tokio::time::Duration::from_secs(1), objects);

        let (objects, _input) = futures::join!(objects, input);
        let _objects = objects.expect("There is a timeout");
    }

    #[tokio::test]
    async fn when_there_are_edges() {
        let child_schema = Uuid::new_v4();
        let a = new_obj(None);
        let b = new_obj(child_schema);
        let c = new_obj(child_schema);

        let a_id = a.0;
        let b_id = b.0;
        let c_id = c.0;

        let first_row_objects = vec![a.clone(), b.clone()];
        let second_row_objects = vec![a.clone(), c.clone()];
        let objects: Vec<_> = vec![a, b, c];
        // Reversed order - to simulate the fact that objects can arrive via network in any order;
        let mut objects_it = objects.clone().into_iter().rev();

        let plan = ViewPlan {
            unfinished_rows: vec![
                Some(UnfinishedRow {
                    fields: hashmap! {
                        "foo".into() => FieldDefinitionSource::Computed {
                            field_type: FieldType::Numeric,
                            computation: crate::sources::ComputationSource::FieldValue {
                                object: b_id,
                                field_path: "".into()
                            }
                        }
                    },
                    filters: None,
                    missing: 2,
                    root_object: a_id,
                    objects: Default::default(),
                    relation_order: vec![a_id],
                }),
                Some(UnfinishedRow {
                    fields: hashmap! {
                        "foo".into() => FieldDefinitionSource::Computed {
                            field_type: FieldType::Numeric,
                            computation: crate::sources::ComputationSource::FieldValue {
                                object: c_id,
                                field_path: "".into()
                            }
                        }
                    },
                    filters: None,
                    missing: 2,
                    root_object: a_id,
                    objects: Default::default(),
                    relation_order: vec![a_id],
                }),
            ],
            missing: hashmap! {
                a_id => vec![0, 1],
                b_id => vec![0],
                c_id => vec![1]
            },
            view: any_view(a_id),
            single_mode: false,
        };

        let (tx, stream) = act(plan);
        pin_mut!(stream);

        // No object arrived, pending
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // First object arrived, but the row is not finished (c)
        assert!(stream.next().now_or_never().is_none());

        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();
        // Second object arrived, but the row is not finished (b)

        assert!(stream.next().now_or_never().is_none());

        // Third object arrived, two row are finished (a)
        tx.send(Ok(objects_it.next().unwrap())).await.unwrap();

        // Stream takes an reversed order from plan.missing.a_id because it uses pop()
        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Join {
                objects: second_row_objects.into_iter().collect(),
                root_object: a_id,
                filters: None,
                fields: hashmap! {
                    "foo".into() => FieldDefinitionSource::Computed {
                        field_type: FieldType::Numeric,
                        computation: crate::sources::ComputationSource::FieldValue {
                            object: c_id,
                            field_path: "".into()
                        }
                    }
                },
                relation_order: vec![a_id],
            },
            "second row"
        );

        assert_eq!(
            stream.next().now_or_never().unwrap().unwrap().unwrap(),
            RowSource::Join {
                objects: first_row_objects.into_iter().collect(),
                root_object: a_id,
                filters: None,
                fields: hashmap! {
                    "foo".into() => FieldDefinitionSource::Computed {
                        field_type: FieldType::Numeric,
                        computation: crate::sources::ComputationSource::FieldValue {
                            object: b_id,
                            field_path: "".into()
                        }
                    }
                },
                relation_order: vec![a_id],
            },
            "first row"
        );
    }

    type TestStream = ObjectBufferedStream<ReceiverStream<Result<(ObjectIdPair, Value)>>>;

    fn new_obj(schema_id: impl Into<Option<Uuid>>) -> (ObjectIdPair, Value) {
        let value = "{}";
        let schema_id = schema_id.into().unwrap_or_else(Uuid::new_v4);
        let pair = ObjectIdPair {
            schema_id,
            object_id: Uuid::new_v4(),
        };
        let value: Value = serde_json::from_str(value).unwrap();
        (pair, value)
    }

    fn act(plan: ViewPlan) -> (Sender<Result<(ObjectIdPair, Value)>>, TestStream) {
        let (tx, rx) = channel(16);
        let rx_stream = ReceiverStream::new(rx);
        let stream = ObjectBufferedStream::new_test(rx_stream, plan);

        (tx, stream)
    }
}
