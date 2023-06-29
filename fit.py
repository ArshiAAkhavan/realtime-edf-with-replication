import json
from typing import List

class Task:
    def __init__(self, name, c, p, d) -> None:
        self.name = name
        self.c = c
        self.p = p
        self.d = d
        self.util = c / p

    def convert(self) -> dict:
        return {
            'name': self.name,
            'c': self.c,
            'p': self.p,
            'd': self.d 
        }


class Processor:
    def __init__(self) -> None:
        self.tasks = []
        self.util = 0


k = int(input('enter number of task replicas: '))
m = int(input('enter number of processors: '))


def get_tasks() -> List[Task]:
    with open("data.json" , 'r') as file:
        data = file.read()
    tasks_json = json.loads(data)

    tasks = []

    for i in tasks_json:
        for j in range(k + 1):
            tasks.append(Task(i['name'], i['c'], i['p'], i['d']))

    tasks.sort(key=lambda x: x.util, reverse=True)    
    return tasks



def get_processors() -> List[Processor]:
    processors = []

    for i in range(m):
        processors.append(Processor())
    
    return processors


    


class Scheduler:
    def __init__(self, tasks: List[Task]) -> None:
        self.tasks = tasks
    
    def write_schedule(self, path: str, processors: List[Processor]):
        schedule = {}
        for i, p in enumerate(processors):
            schedule[i] = [t.convert() for t in p.tasks]
        with open(path, "w") as file:
            json.dump(schedule, file)

            
        

    def get_valid_processors(self, task: Task, processors: List[Processor], schedule_type: str)->List[Processor]:
        valid_processors = []

        for p in processors:
            if p.util + task.util <= 1 and (task.name not in [t.name for t in p.tasks]):
                valid_processors.append(p)

        if schedule_type == 'wfd':
            valid_processors.sort(key=lambda x: x.util)
        elif schedule_type == 'bfd':
            valid_processors.sort(key=lambda x: x.util, reverse=True)

        return valid_processors


    def schedule(self, schedule_type, path): 
        processors = get_processors()
        scheduled_count = 0
        for t in self.tasks:
            valid_processors = self.get_valid_processors(t, processors, schedule_type)
            if len(valid_processors) > 0:
                valid_processors[0].tasks.append(t)
                valid_processors[0].util += t.util
                scheduled_count += 1
        if len(self.tasks) == scheduled_count:
            self.write_schedule(path, processors)
        else:
            print('could not schedule')
        print(schedule_type)
        for p in processors:
            print('util', p.util)
            print('no tasks', len(p.tasks))
            



scheduler = Scheduler(get_tasks())

scheduler.schedule('ffd', 'ffd.json')
scheduler.schedule('wfd', 'wfd.json')
scheduler.schedule('bfd', 'bfd.json')
